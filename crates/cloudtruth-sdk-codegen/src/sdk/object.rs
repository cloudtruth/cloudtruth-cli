use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use syn::{parse_quote, punctuated::Punctuated, token::Comma, Field, Ident, Type};

use super::methods::SdkMethod;

#[derive(Clone)]
pub struct SdkObject {
    name: Ident,
    fields: Punctuated<Field, Comma>,
    methods: Vec<Box<dyn SdkMethod>>,
}

impl SdkObject {
    pub fn new(name: impl AsRef<str>, parent: Option<&SdkObject>) -> Self {
        let mut object = Self {
            name: Ident::new(name.as_ref(), Span::call_site()),
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

    pub fn name(&self) -> &Ident {
        &self.name
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
        self.add_field_ident(Ident::new(name, Span::call_site()), field_type)
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
            name,
            methods,
            fields,
        } = self;
        tokens.extend(quote! {
            pub struct #name {
                #fields
            }
            impl #name {
                #(#methods)*
            }
        });
    }
}
