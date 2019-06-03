use super::types::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use snafu::{ResultExt, Snafu};
use std::convert::TryFrom;

impl RustType {
    pub fn name(&self) -> String {
        match self {
            RustType::PlainEnum { name, .. }
            | RustType::DataEnum { name, .. }
            | RustType::Struct { name, .. } => (name.ident.to_string()),
        }
    }
}

#[derive(Debug, Clone, Snafu)]
pub enum Error {
    #[snafu(display("You've synned at `{}`: {}", token, source))]
    SynError { token: String, source: syn::Error },
}

impl ToTokens for RustType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RustType::PlainEnum { name, variants, attributes, .. } => {
                tokens.append_all(quote! {
                    #[derive(Clone, Copy, Debug, Hash)]
                    #attributes
                    pub enum #name {
                        #(#variants),*
                    }
                });
            }
            RustType::DataEnum { name, variants, attributes, .. } => {
                tokens.append_all(quote! {
                    #[derive(Clone, Debug, Hash)]
                    #attributes
                    pub enum #name {
                        #(#variants),*
                    }
                });
            }
            RustType::Struct { name, fields, attributes, .. } => {
                tokens.append_all(quote! {
                    #[derive(Clone, Debug, Hash)]
                    #attributes
                    pub struct #name {
                        #(#fields),*
                    }
                });
            }
        }
    }
}

impl ToTokens for PlainEnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attributes;
        let name = &self.name;

        tokens.append_all(quote! {
            #attrs
            #name
        });
    }
}

impl ToTokens for DataEnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attributes;
        let name = &self.name;
        let fields = &self.fields;

        match fields {
            DataEnumFields::Named { .. } => {
                tokens.append_all(quote! {
                    #attrs
                    #name {
                        #fields
                    }
                });
            }
            DataEnumFields::Unnamed { .. } => {
                tokens.append_all(quote! {
                    #attrs
                    #name ( #fields )
                });
            }
        }
    }
}

impl ToTokens for DataEnumFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DataEnumFields::Named { fields, .. } => tokens.append_all(quote! { #(#fields),* }),
            DataEnumFields::Unnamed { fields, .. } => tokens.append_all(quote! { #(#fields),* }),
        }
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attributes;
        let name = &self.name;
        let type_name = {
            let name = &self.type_name;
            if self.optional {
                quote! { Option < #name > }
            } else {
                quote! { #name }
            }
        };

        tokens.append_all(quote! {
            #attrs
            #name : #type_name
        });
    }
}

impl<'a> TryFrom<&'a str> for TypeIdent {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use heck::CamelCase;

        let sanitized_ident = value.replace("@", "at_").replace("/", "_").to_camel_case();
        let ident = syn::parse_str(&sanitized_ident)
            .with_context(|| SynError { token: value.to_string() })?;

        Ok(TypeIdent { raw: value.to_string(), ident })
    }
}

impl ToTokens for TypeIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&[&self.ident])
    }
}

impl<'a> TryFrom<&'a str> for FieldName {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use heck::SnakeCase;

        let sanitized_ident = value.replace("@", "at_").replace("/", "_").to_snake_case();
        let ident = syn::parse_str(&sanitized_ident)
            .or_else(|_| syn::parse_str(&format!("r#{}", sanitized_ident)))
            .with_context(|| SynError { token: value.to_string() })?;

        Ok(FieldName { raw: value.to_string(), ident })
    }
}

impl ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&[&self.ident])
    }
}

impl<'a> TryFrom<&'a str> for TypeName {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let ident = syn::parse_str(value).with_context(|| SynError { token: value.to_string() })?;

        Ok(TypeName { raw: value.to_string(), ident })
    }
}

impl ToTokens for TypeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&[&self.ident])
    }
}

impl ToTokens for ContainerAttributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut attrs = vec![];
        if let Some(rename) = &self.rename {
            attrs.push(quote! {
                #[serde(rename = #rename)]
            });
        }
        tokens.append_all(attrs)
    }
}

impl ToTokens for VariantAttributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut attrs = vec![];
        if let Some(rename) = &self.rename {
            attrs.push(quote! {
                #[serde(rename = #rename)]
            });
        }
        tokens.append_all(attrs)
    }
}

impl ToTokens for FieldAttributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut attrs = vec![];
        if let Some(rename) = &self.rename {
            attrs.push(quote! {
                #[serde(rename = #rename)]
            });
        }
        tokens.append_all(attrs)
    }
}
