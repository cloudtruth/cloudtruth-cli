use std::rc::Rc;

use syn::{parse_quote, punctuated::Punctuated, token::Comma, FnArg, Type};

use crate::sdk::SdkObject;

use super::SdkMethod;

#[derive(Clone)]
pub struct SdkChildConstructor {
    name: Rc<str>,
    args: Punctuated<FnArg, Comma>,
}

impl SdkChildConstructor {
    pub fn new(object: &SdkObject) -> Self {
        let name = object.name().clone();
        Self {
            name,
            args: Punctuated::new(),
        }
    }

    pub fn name(&self) -> &Rc<str> {
        &self.name
    }

    pub fn add_arg(&mut self, arg_name: &str, arg_type: Type) {
        self.args.push(parse_quote! [ #arg_name : #arg_type ]);
    }
}

impl SdkMethod for SdkChildConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        let Self { name, args } = self;
        parse_quote! {
            pub fn #name(#args) {

            }
        }
    }
}
