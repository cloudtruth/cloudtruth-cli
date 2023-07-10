use std::rc::Rc;

use syn::parse_quote;

use crate::api::ApiOperation;

use super::SdkMethod;

#[derive(Debug, Clone)]
pub struct SdkApiMethod {
    name: Rc<str>,
    api_op: ApiOperation,
}

impl SdkMethod for SdkApiMethod {
    fn generate_fn(&self) -> syn::ItemFn {
        let SdkApiMethod { name, .. } = self;
        parse_quote! {
            pub fn #(#name)() {

            }
        }
    }
}
