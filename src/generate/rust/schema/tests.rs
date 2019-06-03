use super::types::*;
use proc_macro2::TokenStream;
use quicli::prelude::*;
use quote::quote;
use std::{convert::TryInto, io::Write, path::Path};
use tempfile::NamedTempFile;

#[test]
fn simple_struct() -> CliResult {
    let _ = env_logger::builder().is_test(true).filter(None, log::LevelFilter::Debug).try_init();

    let schemas = vec![RustType::Struct {
        name: "Foo".try_into()?,
        fields: vec![
            StructField {
                name: "one".try_into()?,
                attributes: FieldAttributes { rename: None },
                type_name: "String".try_into()?,
            },
            StructField {
                name: "two".try_into()?,
                attributes: FieldAttributes { rename: None },
                type_name: "std::path::PathBuf".try_into()?,
            },
        ],
        attributes: ContainerAttributes { rename: None },
    }];

    insta::assert_display_snapshot_matches!("simple_struct", generate_rust(&schemas)?);

    Ok(())
}

#[test]
fn plain_enum() -> CliResult {
    let _ = env_logger::builder().is_test(true).filter(None, log::LevelFilter::Debug).try_init();

    let schemas = vec![RustType::PlainEnum {
        name: "Foo".try_into()?,
        variants: vec![PlainEnumVariant {
            name: "Bar".try_into()?,
            attributes: VariantAttributes { rename: None },
        }],
        attributes: ContainerAttributes { rename: None },
    }];

    insta::assert_display_snapshot_matches!("plain_enum", generate_rust(&schemas)?);

    Ok(())
}

#[test]
fn data_enum() -> CliResult {
    let _ = env_logger::builder().is_test(true).filter(None, log::LevelFilter::Debug).try_init();

    let schemas = vec![RustType::DataEnum {
        name: "Foo".try_into()?,
        variants: vec![
            DataEnumVariant {
                name: "Bar".try_into()?,
                attributes: VariantAttributes { rename: None },
                fields: DataEnumFields::Named {
                    fields: vec![
                        StructField {
                            name: "one".try_into()?,
                            attributes: FieldAttributes { rename: None },
                            type_name: "String".try_into()?,
                        },
                        StructField {
                            name: "two".try_into()?,
                            attributes: FieldAttributes { rename: None },
                            type_name: "std::path::PathBuf".try_into()?,
                        },
                    ],
                },
            },
            DataEnumVariant {
                name: "Baz".try_into()?,
                attributes: VariantAttributes { rename: None },
                fields: DataEnumFields::Unnamed {
                    fields: vec!["std::borrow::Cow<'static, str>".try_into()?],
                },
            },
        ],
        attributes: ContainerAttributes { rename: None },
    }];

    insta::assert_display_snapshot_matches!("data_enum", generate_rust(&schemas)?);

    Ok(())
}

fn generate_rust(schemas: &[RustType]) -> Result<String, Error> {
    let schemas: TokenStream = schemas.iter().map(|x| quote! { #x }).collect();

    let mut target = NamedTempFile::new()?;
    writeln!(&mut target, "{}", schemas)?;
    rustfmt(&target.path())?;

    read_file(&target.path())
}

fn rustfmt(path: &Path) -> Result<(), Error> {
    let fmt = std::process::Command::new("rustfmt").arg(&path).status()?;
    if !fmt.success() {
        Err(format_err!("rustfmt failed"))?;
    }
    Ok(())
}
