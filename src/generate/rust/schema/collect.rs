use super::{
    impls,
    types::{
        ContainerAttributes, DataEnumFields, DataEnumVariant, FieldAttributes, PlainEnumVariant,
        RustType, StructField, TypeIdent, TypeName, VariantAttributes,
    },
};
use crate::{
    json_pointer::{JsonPointer, ParseJsonPointerError},
    unref::Unref,
};
use openapiv3::{
    AdditionalProperties, ArrayType, ObjectType, ReferenceOr, SchemaKind, StringType, Type,
};
use snafu::{ResultExt, Snafu};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    convert::{TryFrom, TryInto},
    str::FromStr,
};

type Types = BTreeMap<String, RustType>;

pub fn build(
    schemas: &crate::schemas::Schemas,
    source: &openapiv3::OpenAPI,
) -> Result<Types, Error> {
    let mut res = Types::new();

    for (schema_name, schema) in &schemas.data {
        match &schema.data.schema_kind {
            SchemaKind::Type(Type::Object(obj)) => {
                build_struct_schema(&schema_name.name, &obj, source, &mut res)?;
            }
            x => {
                log::debug!("top level schema type {:?} is unimplemented, skipping!", x);
            }
        }
    }

    Ok(res)
}

/// Returns name of newly inserted schema if there was one
fn build_struct_schema(
    schema_name: &str,
    ObjectType { properties, required, .. }: &ObjectType,
    source: &openapiv3::OpenAPI,
    mut types: &mut Types,
) -> Result<String, Error> {
    let mut fields = Vec::new();
    for (prop_name, prop) in properties {
        let prop = prop.unref(source).context(UnrefError)?;
        let optional = !required.contains(&prop_name);
        fields.push(struct_field(&schema_name, &prop_name, &prop, optional, source, &mut types)?);
    }

    let rename = Some(schema_name.to_owned());
    let t = RustType::Struct {
        name: TypeIdent::try_from(schema_name).context(TemplateError)?,
        fields,
        attributes: ContainerAttributes { rename, ..ContainerAttributes::default() },
    };
    let name = t.name();
    match types.entry(name.clone()) {
        Entry::Occupied(entry) => {
            if *entry.get() != t {
                log::debug!(
                    "{}",
                    Error::DuplicateTypeName {
                        name: name.clone(),
                        previous: format!("{:?}", entry.get()),
                        new: format!("{:?}", t),
                    }
                );
            }
            Ok(name)
        }
        Entry::Vacant(entry) => {
            entry.insert(t);
            Ok(name)
        }
    }
}

/// Returns name of newly inserted schema if there was one
fn build_nested_schema(
    parent_name: &str,
    schema_name: &str,
    schema: &openapiv3::Schema,
    source: &openapiv3::OpenAPI,
    mut types: &mut Types,
) -> Result<String, Error> {
    let name = format!("{}::{}", parent_name, schema_name);

    Ok(match &schema.schema_kind {
        SchemaKind::Type(Type::String(StringType { enumeration, .. })) => {
            if enumeration.is_empty() {
                "String".to_owned()
            } else {
                let new_enum = nested_string_enum(&name, &enumeration)?;
                let new_enum_name = new_enum.name();

                types.insert(new_enum_name.clone(), new_enum);
                new_enum_name
            }
        }
        SchemaKind::Type(Type::Number(_)) => "f64".to_owned(),
        SchemaKind::Type(Type::Integer(_)) => "u64".to_owned(),
        SchemaKind::Type(Type::Object(obj)) => {
            log::trace!("nested object {:?}", obj);
            if obj.properties.is_empty() {
                match &obj.additional_properties {
                    Some(AdditionalProperties::Any(_any)) => {
                        "std::collections::BTreeMap<String, serde_json::Value>".to_owned()
                    }
                    Some(AdditionalProperties::Schema(schema)) => {
                        let value_type_name = match &**schema {
                            ReferenceOr::Reference { reference, .. } => {
                                type_name_from_ref(&reference)?.ident.to_string()
                            }
                            ReferenceOr::Item(schema) => build_nested_schema(
                                &name,
                                "Additional",
                                &schema,
                                source,
                                &mut types,
                            )?,
                        };
                        format!("std::collections::BTreeMap<String, {}>", value_type_name)
                    }
                    None => build_struct_schema(&name, &obj, source, &mut types)?,
                }
            } else {
                build_struct_schema(&name, &obj, source, &mut types)?
            }
        }
        SchemaKind::Type(Type::Array(ArrayType { items, .. })) => {
            let nested_schema_name = match items {
                ReferenceOr::Reference { reference, .. } => {
                    type_name_from_ref(&reference)?.ident.to_string()
                }
                ReferenceOr::Item(schema) => {
                    build_nested_schema(parent_name, schema_name, &schema, source, &mut types)?
                }
            };
            format!("Vec<{}>", nested_schema_name)
        }
        SchemaKind::Type(Type::Boolean { .. }) => "bool".to_owned(),
        SchemaKind::OneOf { one_of } => {
            let name = TypeIdent::try_from(name.as_str()).context(TemplateError)?;
            let n = name.ident.to_string();
            let t = RustType::DataEnum {
                name,
                variants: one_of
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let variant_name = format!("Variant{}", i);
                        let nested_schema_name = match r {
                            ReferenceOr::Reference { reference, .. } => {
                                type_name_from_ref(&reference)?.ident.to_string()
                            }
                            ReferenceOr::Item(schema) => {
                                build_nested_schema(&n, &variant_name, &schema, source, &mut types)?
                            }
                        };
                        Ok(DataEnumVariant {
                            name: TypeIdent::try_from(variant_name.as_str())
                                .context(TemplateError)?,
                            attributes: VariantAttributes::default(),
                            fields: DataEnumFields::Unnamed {
                                fields: vec![TypeName::try_from(nested_schema_name.as_str())
                                    .context(TemplateError)?],
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
                attributes: ContainerAttributes {
                    untagged: true,
                    ..ContainerAttributes::default()
                },
            };
            types.insert(n.clone(), t);
            n
        }
        x => {
            // return Err(Error::Unimplemented { info: format!("schema type {:?}", x) })
            log::debug!("schema type {:?} is unimplemented!", x);
            "serde_json::Value".to_owned()
        }
    })
}

fn type_name_from_ref(r: &str) -> Result<TypeIdent, Error> {
    let reference = JsonPointer::from_str(&r).context(InvalidJsonPointer)?;

    match &reference.components()[..] {
        &["components", "schemas", name] => Ok(TypeIdent::try_from(name).context(TemplateError)?),
        _ => return Err(Error::UnsupportedReference { reference: reference.clone() }),
    }
}

fn struct_field(
    parent_name: &str,
    name: &str,
    input: &openapiv3::Schema,
    optional: bool,
    source: &openapiv3::OpenAPI,
    mut types: &mut Types,
) -> Result<StructField, Error> {
    let type_name = build_nested_schema(parent_name, name, &input, source, &mut types)?;

    Ok(StructField {
        name: name.try_into().context(TemplateError)?,
        type_name: TypeName::try_from(type_name.as_str()).context(TemplateError)?,
        attributes: FieldAttributes { rename: Some(name.to_string()) },
        optional,
    })
}

fn nested_string_enum(name: &str, variants: &[String]) -> Result<RustType, Error> {
    Ok(RustType::PlainEnum {
        name: TypeIdent::try_from(name).context(TemplateError)?,
        variants: variants
            .iter()
            .map(|t| {
                Ok(PlainEnumVariant {
                    name: TypeIdent::try_from(t.as_str()).context(TemplateError)?,
                    attributes: VariantAttributes { rename: Some(t.to_owned()) },
                })
            })
            .collect::<Result<Vec<_>, Error>>()?,
        attributes: ContainerAttributes::default(),
    })
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("Error following reference: {}", source))]
    UnrefError { source: crate::unref::Error },
    #[snafu(display("Error building Rust type: {}", source))]
    TemplateError { source: impls::Error },
    #[snafu(display(
        "Duplicate type name `{}`. Previous definition:\n{}\nNew definition:\n{}",
        name,
        previous,
        new
    ))]
    DuplicateTypeName { name: String, previous: String, new: String },
    #[snafu(display("Invalid JSON pointer as reference"))]
    InvalidJsonPointer { source: ParseJsonPointerError },
    #[snafu(display("Reference location `{:?}` currently not supported", reference))]
    UnsupportedReference { reference: JsonPointer },
    #[snafu(display("Sorry, {} is not implemented", info))]
    Unimplemented { info: String },
}
