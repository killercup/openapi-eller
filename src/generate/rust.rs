use crate::Unref;
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use snafu::{ResultExt, Snafu};

fn field_name(s: &str) -> Result<syn::Ident, Error> {
    syn::parse_str(&format!("r#{}", s.replace("@", "at_").to_snake_case())).with_context(|| SynError {
        token: s.to_string(),
    })
}

fn type_name(s: &str) -> Result<syn::Type, Error> {
    syn::parse_str(&s).with_context(|| SynError {
        token: s.to_string(),
    })
}

fn schema_kind(i: openapiv3::SchemaKind, source: &openapiv3::OpenAPI) -> Result<String, Error> {
    Ok(match i {
        openapiv3::SchemaKind::Type(openapiv3::Type::String(x)) => {
            if !x.enumeration.is_empty() {
                log::debug!("enum schemas are hard: `{:?}`", x);
            }
            "String".to_owned()
        }
        openapiv3::SchemaKind::Type(openapiv3::Type::Number(_)) => "f64".to_owned(),
        openapiv3::SchemaKind::Type(openapiv3::Type::Integer(_)) => "u64".to_owned(),
        openapiv3::SchemaKind::Type(openapiv3::Type::Object(obj)) => {
            log::debug!("nested schemas are hard: `{:?}`", obj);
            format!("std::collections::HashMap<String, Box<dyn std::any::Any>>")
        }
        openapiv3::SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
            items, ..
        })) => {
            let item_type =
                schema_kind(items.unref(source).context(UnrefError)?.schema_kind, source)?;
            format!("Vec<{}>", item_type)
        }
        openapiv3::SchemaKind::Type(openapiv3::Type::Boolean { .. }) => "bool".to_owned(),
        x => return Err(Error::Unimplemented {
            info: format!("schema type {:?}", x),
        }),
    })
}

pub fn types(
    schemas: &crate::schemas::Schemas,
    source: &openapiv3::OpenAPI,
) -> Result<TokenStream, Error> {
    let res = schemas
        .data
        .iter()
        .flat_map(|(struct_name, schema)| {
            let struct_name = type_name(&struct_name.name)?;
            match &schema.data.schema_kind {
                openapiv3::SchemaKind::Type(openapiv3::Type::Object(obj)) => {
                    let fields = obj
                        .properties
                        .iter()
                        .map(|(name, props)| {
                            let field = props.unref(source).context(UnrefError)?;

                            let name = field_name(&name)?;
                            let field_type = type_name(&schema_kind(field.schema_kind, source)?)?;

                            Ok(quote! {
                                #name : #field_type
                            })
                        })
                        .collect::<Result<Vec<TokenStream>, Error>>()?;

                    Ok(quote! {
                        struct #struct_name {
                            #( #fields ),*
                        }
                    })
                }
                openapiv3::SchemaKind::Type(x) => Err(Error::Unimplemented {
                    info: format!("schema type {:?}", x),
                }),
                x => Err(Error::Unimplemented {
                    info: format!("schema type {:?}", x),
                }),
            }
        })
        .collect::<TokenStream>();

    let res = quote! { #(#res)* };
    Ok(res.into())
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("You've synned at `{}`: {}", token, source))]
    SynError { token: String, source: syn::Error },
    #[snafu(display("Error following reference: {}", source))]
    UnrefError { source: crate::unref::Error },
    #[snafu(display("Sorry, {} is not implemented", info))]
    Unimplemented { info: String },
}
