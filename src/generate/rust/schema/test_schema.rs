const SIMPLE_SCHEMA: &str = "
openapi: 3.0.1
info: { title: Foo, version: 0.1.0 }
paths: {}
components:
    schemas:
        Foo:
            type: object
            properties:
                bar:
                    type: string
";

#[test]
fn simple_schema() {
    let api: openapiv3::OpenAPI = serde_yaml::from_str(SIMPLE_SCHEMA).unwrap();

    insta::assert_display_snapshot_matches!(
        "simple_schema",
        crate::generate::rust::types(&api).unwrap()
    );
}

const NESTED_SCHEMA: &str = "
openapi: 3.0.1
info: { title: Foo, version: 0.1.0 }
paths: {}
components:
    schemas:
        Foo:
            type: object
            properties:
                bar:
                    type: object
                    properties:
                        baz:
                            type: string
";

#[test]
fn nested_schema() {
    let api: openapiv3::OpenAPI = serde_yaml::from_str(NESTED_SCHEMA).unwrap();

    insta::assert_display_snapshot_matches!(
        "nested_schema",
        crate::generate::rust::types(&api).unwrap()
    );
}

const STRING_ENUM: &str = "
openapi: 3.0.1
info: { title: Foo, version: 0.1.0 }
paths: {}
components:
    schemas:
        Foo:
            type: object
            properties:
                bar:
                    type: string
                    enum:
                        - lorem
                        - ipsum
                        - dolor
";

#[test]
fn string_enum() {
    let api: openapiv3::OpenAPI = serde_yaml::from_str(STRING_ENUM).unwrap();

    insta::assert_display_snapshot_matches!(
        "string_enum",
        crate::generate::rust::types(&api).unwrap()
    );
}

const SCHEMA_REF: &str = "
openapi: 3.0.1
info: { title: Foo, version: 0.1.0 }
paths: {}
components:
    schemas:
        Foo:
            type: object
            properties:
                bar:
                    $ref: '#/components/schemas/Bar'
        Bar:
            type: object
            properties:
                baz:
                    type: integer
";

// FIXME: Ref schema name spaces!
#[test]
fn schema_ref() {
    let api: openapiv3::OpenAPI = serde_yaml::from_str(SCHEMA_REF).unwrap();

    insta::assert_display_snapshot_matches!(
        "schema_ref",
        crate::generate::rust::types(&api).unwrap()
    );
}
