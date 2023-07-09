use std::borrow::Cow;

use syn::parse_quote;

use crate::api::ApiOperation;

use super::SdkMethod;

#[derive(Debug, Clone)]
pub struct SdkApiMethod<'a> {
    name: Cow<'a, str>,
    api_op: ApiOperation,
}

impl<'a> SdkMethod for SdkApiMethod<'a> {
    fn generate_fn(&self) -> syn::ItemFn {
        let SdkApiMethod { name, .. } = self;
        parse_quote! {
            pub fn #(#name)() {

            }
        }
    }
}
