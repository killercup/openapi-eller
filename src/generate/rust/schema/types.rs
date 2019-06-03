#[derive(Debug, Clone)]
pub enum RustType {
    PlainEnum { name: TypeIdent, variants: Vec<PlainEnumVariant>, attributes: ContainerAttributes },
    DataEnum { name: TypeIdent, variants: Vec<DataEnumVariant>, attributes: ContainerAttributes },
    Struct { name: TypeIdent, fields: Vec<StructField>, attributes: ContainerAttributes },
}

#[derive(Debug, Clone)]
pub struct PlainEnumVariant {
    pub name: TypeIdent,
    pub attributes: VariantAttributes,
}

#[derive(Debug, Clone)]
pub struct DataEnumVariant {
    pub name: TypeIdent,
    pub attributes: VariantAttributes,
    pub fields: DataEnumFields,
}

#[derive(Debug, Clone)]
pub enum DataEnumFields {
    Named { fields: Vec<StructField> },
    Unnamed { fields: Vec<TypeName> },
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: FieldName,
    pub attributes: FieldAttributes,
    pub type_name: TypeName,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct TypeIdent {
    pub raw: String,
    pub ident: syn::Ident,
}

#[derive(Debug, Clone)]
pub struct FieldName {
    pub raw: String,
    pub ident: syn::Ident,
}

#[derive(Debug, Clone)]
pub struct TypeName {
    pub raw: String,
    pub ident: syn::Type,
}

#[derive(Debug, Clone)]
pub struct ContainerAttributes {
    pub rename: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VariantAttributes {
    pub rename: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldAttributes {
    pub rename: Option<String>,
}
