use super::{
    impls,
    types::{
        ContainerAttributes, FieldAttributes, PlainEnumVariant, RustType, StructField, TypeIdent,
        TypeName, VariantAttributes,
    },
};
use crate::unref::Unref;
use openapiv3::{ArrayType, ObjectType, SchemaKind, StringType, Type};
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

type Types = HashMap<String, RustType>;

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
        fields.push(struct_field(&schema_name, &prop_name, &prop, source, &mut types)?);
    }

    let rename = Some(schema_name.to_owned());
    let t = RustType::Struct {
        name: TypeIdent::try_from(schema_name).context(TemplateError)?,
        fields,
        attributes: ContainerAttributes { rename },
    };
    let name = t.name();
    types.insert(name.clone(), t);
    Ok(name)
}

/// Returns name of newly inserted schema if there was one
fn build_nested_schema(
    parent_name: &str,
    schema_name: &str,
    schema: &openapiv3::Schema,
    source: &openapiv3::OpenAPI,
    mut types: &mut Types,
) -> Result<String, Error> {
    Ok(match &schema.schema_kind {
        SchemaKind::Type(Type::String(StringType { enumeration, .. })) => {
            if enumeration.is_empty() {
                "String".to_owned()
            } else {
                let new_enum =
                    nested_string_enum(&format!("{} {}", parent_name, schema_name), &enumeration)?;
                let new_enum_name = new_enum.name();

                types.insert(new_enum_name.clone(), new_enum);
                new_enum_name
            }
        }
        SchemaKind::Type(Type::Number(_)) => "f64".to_owned(),
        SchemaKind::Type(Type::Integer(_)) => "u64".to_owned(),
        SchemaKind::Type(Type::Object(obj)) => {
            build_struct_schema(schema_name, &obj, source, &mut types)?
        }
        SchemaKind::Type(Type::Array(ArrayType { items, .. })) => {
            // TODO: Figure out whether this is an existing schema and just
            // insert its type name here
            let schema = items.unref(source).context(UnrefError)?;
            let nested_schema_name =
                build_nested_schema(parent_name, schema_name, &schema, source, &mut types)?;
            format!("Vec<{}>", nested_schema_name)
        }
        SchemaKind::Type(Type::Boolean { .. }) => "bool".to_owned(),
        x => {
            // return Err(Error::Unimplemented { info: format!("schema type {:?}", x) })
            log::debug!("schema type {:?} is unimplemented!", x);
            "serde_json::Value".to_owned()
        }
    })
}

fn struct_field(
    parent_name: &str,
    name: &str,
    input: &openapiv3::Schema,
    source: &openapiv3::OpenAPI,
    mut types: &mut Types,
) -> Result<StructField, Error> {
    let type_name = build_nested_schema(parent_name, name, &input, source, &mut types)?;

    Ok(StructField {
        name: name.try_into().context(TemplateError)?,
        type_name: TypeName::try_from(type_name.as_str()).context(TemplateError)?,
        attributes: FieldAttributes { rename: Some(name.to_owned()) },
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
        attributes: ContainerAttributes { rename: None },
    })
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("Error following reference: {}", source))]
    UnrefError { source: crate::unref::Error },
    #[snafu(display("Error building Rust type: {}", source))]
    TemplateError { source: impls::Error },
    #[snafu(display("Sorry, {} is not implemented", info))]
    Unimplemented { info: String },
}
