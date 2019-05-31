use im::{vector, Vector};
use snafu::{OptionExt, Snafu};
use std::fmt;

#[derive(Clone, Default, Debug)]
pub struct Schemas {
    data: std::collections::HashMap<Identifier, Schema>,
}

impl Schemas {
    pub fn keys(&self) -> impl Iterator<Item = String> + '_ {
        self.data.keys().map(|x| x.to_string())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Identifier {
    namespace: Vector<String>,
    name: String,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for name in &self.namespace {
            write!(f, "{}.", name)?;
        }
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
struct Schema {
    id: Identifier,
    data: openapiv3::Schema,
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("Cannot infer name from namespace"))]
    CannotInferNameFromNamespace {
        namespace: Vector<String>,
        schema: openapiv3::Schema,
    },
}

pub fn collect_schemas(input: &openapiv3::OpenAPI) -> Result<Schemas, Error> {
    let mut store = Schemas::default();
    input.collect_schema(&mut store, Vector::new())?;
    Ok(store)
}

trait Visitor {
    fn collect_schema(&self, store: &mut Schemas, namespace: Vector<String>) -> Result<(), Error>;
}

impl Visitor for openapiv3::OpenAPI {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        for (path, item) in &self.paths {
            item.collect_schema(&mut store, namespace.clone() + vector![path.clone()])?;
        }
        if let Some(components) = &self.components {
            components.collect_schema(&mut store, namespace.clone())?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::PathItem {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        if let Some(op) = &self.get {
            op.responses.collect_schema(&mut store, namespace.clone())?;
        }
        if let Some(op) = &self.put {
            op.responses.collect_schema(&mut store, namespace.clone())?;
        }
        if let Some(op) = &self.post {
            op.responses.collect_schema(&mut store, namespace.clone())?;
        }
        if let Some(op) = &self.delete {
            op.responses.collect_schema(&mut store, namespace.clone())?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Responses {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        if let Some(res) = &self.default {
            res.collect_schema(&mut store, namespace.clone())?;
        }

        for (name, response) in &self.responses {
            let namespace = namespace.clone() + vector![name.clone()];
            response.collect_schema(&mut store, namespace)?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Response {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        for (name, content_type) in &self.content {
            content_type.collect_schema(&mut store, namespace.clone() + vector![name.clone()])?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::MediaType {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        if let Some(res) = &self.schema {
            res.collect_schema(&mut store, namespace.clone())?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Schema {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        let mut inner_namespace = namespace.clone();
        let name = inner_namespace
            .pop_back()
            .with_context(|| CannotInferNameFromNamespace {
                namespace: namespace.clone(),
                schema: self.clone(),
            })?;
        let ident = Identifier {
            namespace: inner_namespace,
            name,
        };

        store.data.insert(
            ident.clone(),
            Schema {
                id: ident.clone(),
                data: self.clone(),
            },
        );

        use openapiv3::SchemaKind::*;
        match &self.schema_kind {
            OneOf { one_of } => {
                for schema in one_of {
                    schema.collect_schema(&mut store, namespace.clone())?;
                }
            }
            AllOf { all_of } => {
                for schema in all_of {
                    schema.collect_schema(&mut store, namespace.clone())?;
                }
            }
            AnyOf { any_of } => {
                for schema in any_of {
                    schema.collect_schema(&mut store, namespace.clone())?;
                }
            }
            _ => {}
        };

        Ok(())
    }
}

impl<T: Visitor> Visitor for openapiv3::ReferenceOr<T> {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        namespace: Vector<String>,
    ) -> Result<(), Error> {
        if let openapiv3::ReferenceOr::Item(item) = self {
            item.collect_schema(&mut store, namespace.clone())?;
        } else {
            log::debug!("reference schema collection not yet implemented (ReferenceOr)");
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Components {
    fn collect_schema(&self, store: &mut Schemas, namespace: Vector<String>) -> Result<(), Error> {
        for (name, schema) in &self.schemas {
            let ident = Identifier {
                namespace: namespace.clone(),
                name: name.clone(),
            };
            if let openapiv3::ReferenceOr::Item(schema) = schema {
                store.data.insert(
                    ident.clone(),
                    Schema {
                        id: ident,
                        data: schema.clone(),
                    },
                );
            } else {
                log::debug!("reference schema collection not yet implemented (Components)");
            }
        }
        Ok(())
    }
}
