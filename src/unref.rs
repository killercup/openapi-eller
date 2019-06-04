use crate::json_pointer::{JsonPointer, ParseJsonPointerError};
use openapiv3::{OpenAPI, ReferenceOr, Schema as JsonSchema};
use snafu::{OptionExt, ResultExt, Snafu};
use std::{str::FromStr, sync::Arc};

pub trait Unref<T> {
    fn unref<'b>(&self, all: &'b OpenAPI) -> Result<Arc<T>, Error>;
}

impl Unref<JsonSchema> for ReferenceOr<JsonSchema> {
    fn unref<'b>(&self, all: &'b OpenAPI) -> Result<Arc<JsonSchema>, Error> {
        match self {
            ReferenceOr::Item(x) => Ok(x.clone()),
            ReferenceOr::Reference { reference, .. } => {
                let pointer = JsonPointer::from_str(&reference).context(InvalidJsonPointer)?;
                resolve(&pointer, all)
            }
        }
    }
}

fn resolve<'a, 'b>(reference: &'a JsonPointer, all: &'b OpenAPI) -> Result<Arc<JsonSchema>, Error> {
    match &reference.components()[..] {
        &["components", "schemas", name] => {
            log::trace!("nice, {:?}", &reference.components()[2..]);

            let refd_schema = all
                .components
                .as_ref()
                .with_context(|| NoComponentsDefinedInSchema { reference: reference.clone() })?
                .schemas
                .get(name)
                .with_context(|| ReferenceNotFound { reference: reference.clone() })?;

            Ok(refd_schema.unref(all)?.clone())
        }
        _ => return Err(Error::UnsupportedReference { reference: reference.clone() }),
    }
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("Invalid JSON pointer as reference"))]
    InvalidJsonPointer { source: ParseJsonPointerError },
    #[snafu(display("Reference location `{:?}` currently not supported", reference))]
    UnsupportedReference { reference: JsonPointer },
    #[snafu(display(
        "Referenced item `{:?}` could not be found: No components defined in schema",
        reference
    ))]
    NoComponentsDefinedInSchema { reference: JsonPointer },
    #[snafu(display("Referenced item `{:?}` could not be found", reference))]
    ReferenceNotFound { reference: JsonPointer },
}
