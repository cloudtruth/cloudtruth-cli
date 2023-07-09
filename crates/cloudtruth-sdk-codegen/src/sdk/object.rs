use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;

use super::methods::SdkMethod;

pub struct SdkObject<'a> {
    name: Cow<'a, str>,
    methods: Vec<Box<dyn SdkMethod>>,
    children: Vec<SdkObject<'a>>,
}

impl<'a> SdkObject<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        let name = name.into();
        Self {
            name,
            methods: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn methods(&self) -> &[Box<dyn SdkMethod>] {
        &self.methods
    }

    pub fn add_method(&mut self, method: impl SdkMethod + 'static) -> &mut Self {
        self.methods.push(Box::new(method));
        self
    }

    pub fn children(&self) -> &[SdkObject<'a>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [SdkObject<'a>] {
        &mut self.children
    }

    pub fn add_child(&mut self, child: SdkObject<'a>) {
        self.children.push(child);
    }
}

impl<'a> ToTokens for SdkObject<'a> {
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
