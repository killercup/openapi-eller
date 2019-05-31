use crate::Unref;
use im::{vector, Vector};
use snafu::{OptionExt, ResultExt, Snafu};
use std::{fmt, sync::Arc};

pub fn collect_schemas(input: &openapiv3::OpenAPI) -> Result<Schemas, Error> {
    let mut store = Schemas::default();
    let input = Arc::new(input.clone());
    input.collect_schema(
        &mut store,
        &VisitorContext {
            namespace: Vector::new(),
            all: Arc::clone(&input),
        },
    )?;
    Ok(store)
}

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
    #[snafu(display("Cannot resolve reference"))]
    CannotUnref { source: crate::unref::Error },
}

trait Visitor {
    fn collect_schema(&self, store: &mut Schemas, context: &VisitorContext) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
struct VisitorContext {
    namespace: Vector<String>,
    all: Arc<openapiv3::OpenAPI>,
}

impl VisitorContext {
    pub fn sub_namespace(&self, path: &str) -> Self {
        VisitorContext {
            namespace: self.namespace.clone() + vector![path.to_string()],
            all: Arc::clone(&self.all),
        }
    }

    pub fn maybe_replace_namespace(&self, path: Option<&String>) -> Self {
        if let Some(path) = path {
            VisitorContext {
                namespace: vector![path.to_string()],
                all: Arc::clone(&self.all),
            }
        } else {
            self.clone()
        }
    }
}

impl Visitor for openapiv3::OpenAPI {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        for (path, item) in &self.paths {
            item.collect_schema(&mut store, &context.sub_namespace(path))?;
        }
        if let Some(components) = &self.components {
            components.collect_schema(&mut store, context)?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::ReferenceOr<openapiv3::Schema> {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        self.unref(&context.all)
            .context(CannotUnref)?
            .collect_schema(&mut store, context)?;
        Ok(())
    }
}

impl Visitor for openapiv3::ReferenceOr<openapiv3::PathItem> {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let openapiv3::ReferenceOr::Item(item) = self {
            item.collect_schema(&mut store, context)?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::ReferenceOr<openapiv3::Response> {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let openapiv3::ReferenceOr::Item(item) = self {
            item.collect_schema(&mut store, context)?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::ReferenceOr<openapiv3::MediaType> {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let openapiv3::ReferenceOr::Item(item) = self {
            item.collect_schema(&mut store, context)?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::PathItem {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let Some(op) = &self.get {
            op.responses.collect_schema(
                &mut store,
                &context.maybe_replace_namespace(op.operation_id.as_ref()),
            )?;
        }
        if let Some(op) = &self.put {
            op.responses.collect_schema(
                &mut store,
                &context.maybe_replace_namespace(op.operation_id.as_ref()),
            )?;
        }
        if let Some(op) = &self.post {
            op.responses.collect_schema(
                &mut store,
                &context.maybe_replace_namespace(op.operation_id.as_ref()),
            )?;
        }
        if let Some(op) = &self.delete {
            op.responses.collect_schema(
                &mut store,
                &context.maybe_replace_namespace(op.operation_id.as_ref()),
            )?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Responses {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let Some(res) = &self.default {
            res.collect_schema(&mut store, context)?;
        }

        for (name, response) in &self.responses {
            response.collect_schema(&mut store, &context.sub_namespace(name))?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Response {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        for (name, content_type) in &self.content {
            content_type.collect_schema(&mut store, &context.sub_namespace(name))?;
        }
        Ok(())
    }
}

impl Visitor for openapiv3::MediaType {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        if let Some(res) = &self.schema {
            // Skip top-level ref-schemas
            if let openapiv3::ReferenceOr::Item(item) = res {
                item.collect_schema(&mut store, context)?;
            }
        }
        Ok(())
    }
}

impl Visitor for openapiv3::Schema {
    fn collect_schema(
        &self,
        mut store: &mut Schemas,
        context: &VisitorContext,
    ) -> Result<(), Error> {
        let mut inner_namespace = context.namespace.clone();
        let name = inner_namespace
            .pop_back()
            .with_context(|| CannotInferNameFromNamespace {
                namespace: context.namespace.clone(),
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
                    schema.collect_schema(&mut store, context)?;
                }
            }
            AllOf { all_of } => {
                for schema in all_of {
                    schema.collect_schema(&mut store, context)?;
                }
            }
            AnyOf { any_of } => {
                for schema in any_of {
                    schema.collect_schema(&mut store, context)?;
                }
            }
            _ => {}
        };

        Ok(())
    }
}

impl Visitor for openapiv3::Components {
    fn collect_schema(&self, store: &mut Schemas, context: &VisitorContext) -> Result<(), Error> {
        for (name, schema) in &self.schemas {
            let ident = Identifier {
                namespace: context.namespace.clone(),
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
