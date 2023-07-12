use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use syn::{parse_quote, punctuated::Punctuated, token::Comma, Field, Ident, Type};

use crate::ident;
use crate::names;

use super::methods::SdkMethod;

#[derive(Clone)]
pub struct SdkObject {
    name: Rc<Ident>,
    type_name: Rc<Ident>,
    fields: Punctuated<Field, Comma>,
    methods: Vec<Box<dyn SdkMethod>>,
}

impl SdkObject {
    pub fn new(url_path: impl AsRef<str>, parent: Option<&SdkObject>) -> Self {
        let url_path = url_path.as_ref();
        let mut object = Self {
            type_name: Rc::new(ident!(names::convert_url_to_type_name(url_path))),
            name: Rc::new(ident!(url_path
                .split('/')
                .map(names::trim_path_var_brackets)
                .last()
                .unwrap())),
            methods: Vec::new(),
            fields: Punctuated::new(),
        };
        // add parent fields
        if let Some(parent) = parent {
            for field in parent.fields() {
                object.add_field_ident(field.ident.clone().unwrap(), field.ty.clone());
            }
        }
        object
    }

    /// Returns the name as-is (usually snake_case but depends on naming convention of API)
    pub fn name(&self) -> &Rc<Ident> {
        &self.name
    }

    /// Returns the name to be used in a Rust type (i.e. PascalCase)
    pub fn type_name(&self) -> &Rc<Ident> {
        &self.type_name
    }

    pub fn methods(&self) -> &[Box<dyn SdkMethod>] {
        &self.methods
    }

    pub fn add_method(&mut self, method: impl SdkMethod + 'static) -> &mut Self {
        self.methods.push(Box::new(method));
        self
    }

    pub fn fields(&self) -> &Punctuated<Field, Comma> {
        &self.fields
    }

    pub fn add_field(&mut self, name: &str, field_type: Type) -> &mut Self {
        self.add_field_ident(ident!(name), field_type)
    }

    pub fn add_field_ident(&mut self, ident: Ident, field_type: Type) -> &mut Self {
        self.fields.push(Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(ident),
            colon_token: parse_quote![:],
            ty: field_type,
        });
        self
    }
}

impl ToTokens for SdkObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SdkObject {
            type_name,
            methods,
            fields,
            ..
        } = self;
        tokens.extend(quote! {
            pub struct #type_name {
                #fields
            }
            impl #type_name {
                #(#methods)*
            }
        });
    }
}
