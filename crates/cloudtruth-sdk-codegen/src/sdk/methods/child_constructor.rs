use std::borrow::Cow;

use syn::parse_quote;

use crate::sdk::SdkObject;

use super::SdkMethod;

pub struct SdkChildConstructor<'a> {
    name: Cow<'a, str>,
    object: &'a SdkObject<'a>,
}

impl<'a> SdkMethod for SdkChildConstructor<'a> {
    fn generate_fn(&self) -> syn::ItemFn {
        let SdkChildConstructor { name, .. } = self;
        parse_quote! {
            pub fn #(#name)() {

            }
        }
    }
}
