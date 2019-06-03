use super::{
    impls,
    types::{ContainerAttributes, FieldAttributes, RustType, StructField, TypeIdent, TypeName},
};
use crate::unref::Unref;
use openapiv3::{ArrayType, ObjectType, SchemaKind, Type};
use snafu::{ResultExt, Snafu};
use std::convert::{TryFrom, TryInto};

pub fn build(
    schemas: &crate::schemas::Schemas,
    source: &openapiv3::OpenAPI,
) -> Result<Vec<RustType>, Error> {
    let mut res = Vec::new();

    for (schema_name, schema) in &schemas.data {
        match &schema.data.schema_kind {
            SchemaKind::Type(Type::Object(ObjectType { properties, required, .. })) => {
                let mut fields = Vec::new();
                for (prop_name, prop) in properties {
                    let prop = prop.unref(source).context(UnrefError)?;
                    fields.push(struct_field(prop_name, &prop, source, &mut res)?);
                }

                let rename = Some(schema_name.name.clone());
                let t = RustType::Struct {
                    name: TypeIdent::try_from(schema_name.name.as_str()).context(TemplateError)?,
                    fields,
                    attributes: ContainerAttributes { rename },
                };
                res.push(t);
            }
            x => {
                log::debug!("top level schema type {:?} is unimplemented, skipping!", x);
            }
        }
    }

    Ok(res)
}

fn struct_field(
    name: &str,
    input: &openapiv3::Schema,
    source: &openapiv3::OpenAPI,
    mut types: &mut Vec<RustType>,
) -> Result<StructField, Error> {
    let type_name = match &input.schema_kind {
        SchemaKind::Type(Type::String(x)) => {
            if x.enumeration.is_empty() {
                "String".to_owned()
            } else {
                log::debug!("enum schemas are hard, skipping. `{:?}`", x);
                "String".to_owned()
            }
        }
        SchemaKind::Type(Type::Number(_)) => "f64".to_owned(),
        SchemaKind::Type(Type::Integer(_)) => "u64".to_owned(),
        SchemaKind::Type(Type::Object(obj)) => {
            log::debug!("nested schemas are hard, skipping. `{:?}`", obj);
            format!("std::collections::HashMap<String, Box<dyn std::any::Any>>")
        }
        SchemaKind::Type(Type::Array(ArrayType { items, .. })) => {
            format!("Vec<{}>", "Box<dyn std::any::Any>")
        }
        SchemaKind::Type(Type::Boolean { .. }) => "bool".to_owned(),
        x => {
            // return Err(Error::Unimplemented { info: format!("schema type {:?}", x) })
            log::debug!("schema type {:?} is unimplemented!", x);
            "Box<dyn std::any::Any>".to_owned()
        }
    };

    Ok(StructField {
        name: name.try_into().context(TemplateError)?,
        type_name: TypeName::try_from(type_name.as_str()).context(TemplateError)?,
        attributes: FieldAttributes { rename: Some(name.to_owned()) },
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
