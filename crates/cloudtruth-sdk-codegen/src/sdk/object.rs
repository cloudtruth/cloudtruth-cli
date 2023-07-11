use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::rc::Rc;

use super::methods::SdkMethod;

#[derive(Clone)]
pub struct SdkObject {
    name: Rc<str>,
    methods: Vec<Box<dyn SdkMethod>>,
}

impl SdkObject {
    pub fn new(name: impl Into<Rc<str>>) -> Self {
        let name = name.into();
        Self {
            name,
            methods: Vec::new(),
        }
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
}

impl ToTokens for SdkObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SdkObject {
            name: type_name,
            methods,
            ..
        } = self;
        tokens.extend(quote! {
            pub struct #type_name {
            }
            impl #type_name {
                #(#methods)*
            }
        });
    }
}
