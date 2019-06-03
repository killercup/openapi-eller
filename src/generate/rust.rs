use snafu::{ResultExt, Snafu};

mod schema;
mod types;

pub fn types(source: &openapiv3::OpenAPI) -> Result<String, Error> {
    use crate::collect_schemas;
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::io::Write;
    use tempfile::NamedTempFile;

    let schemas = collect_schemas(source).context(SchemaError)?;
    let rust_types = schema::collect::build(&schemas, source).context(BuildRustError)?;
    let rust_types: TokenStream = rust_types.iter().map(|x| quote! { #x }).collect();

    let mut target = NamedTempFile::new().map_err(|e| Error::Rustfmt { info: e.to_string() })?;
    writeln!(&mut target, "{}", rust_types).map_err(|e| Error::Rustfmt { info: e.to_string() })?;

    let fmt = std::process::Command::new("rustfmt")
        .arg(&target.path())
        .status()
        .map_err(|e| Error::Rustfmt { info: e.to_string() })?;
    if !fmt.success() {
        Err(Error::Rustfmt { info: "rustfmt exited unsuccessfully".to_string() })?;
    }

    Ok(std::fs::read_to_string(&target.path())
        .map_err(|e| Error::Rustfmt { info: e.to_string() })?)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error collecting schema information: {}", source))]
    SchemaError { source: crate::schemas::Error },
    #[snafu(display("Error collecting schema information: {}", source))]
    BuildRustError { source: schema::collect::Error },
    #[snafu(display("Error executing rustfmt: {}", info))]
    Rustfmt { info: String },
    #[snafu(display("Sorry, {} is not implemented", info))]
    Unimplemented { info: String },
}
