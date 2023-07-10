use std::rc::Rc;

use syn::parse_quote;

use crate::sdk::SdkObject;

use super::SdkMethod;

#[derive(Clone)]
pub struct SdkChildConstructor {
    name: Rc<str>,
    object: Rc<SdkObject>,
}

impl SdkMethod for SdkChildConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        let SdkChildConstructor { name, .. } = self;
        parse_quote! {
            pub fn #(#name)() {

            }
        }
    }
}
