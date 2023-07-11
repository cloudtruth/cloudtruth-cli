use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::rc::Rc;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Field, Ident, Type};

use super::methods::SdkMethod;

#[derive(Clone)]
pub struct SdkObject {
    name: Rc<str>,
    fields: Punctuated<Field, Comma>,
    methods: Vec<Box<dyn SdkMethod>>,
}

impl SdkObject {
    pub fn new(name: impl Into<Rc<str>>) -> Self {
        let name = name.into();
        let mut object = Self {
            name,
            methods: Vec::new(),
            fields: Punctuated::new(),
        };
        object.add_field("client", parse_quote![Arc<Client>]);
        object
    }

    pub fn name(&self) -> &Rc<str> {
        &self.name
    }

    pub fn methods(&self) -> &[Box<dyn SdkMethod>] {
        &self.methods
    }

    pub fn add_method(&mut self, method: impl SdkMethod + 'static) -> &mut Self {
        self.methods.push(Box::new(method));
        self
    }

    pub fn add_field(&mut self, name: &str, field_type: Type) -> &mut Self {
        self.fields.push(Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(Ident::new(name, Span::call_site())),
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
