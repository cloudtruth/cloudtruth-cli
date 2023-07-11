use std::rc::Rc;

use syn::parse_quote;

use crate::api::ApiOperation;

use super::SdkMethod;

#[derive(Debug, Clone)]
pub struct SdkApiMethod {
    api_op: Rc<ApiOperation>,
}

impl SdkApiMethod {
    pub fn new(api_op: impl Into<Rc<ApiOperation>>) -> Self {
        let api_op = api_op.into();
        SdkApiMethod { api_op }
    }

    fn name(&self) -> &Rc<str> {
        self.api_op.operation_id().unwrap()
    }
}

impl SdkMethod for SdkApiMethod {
    fn generate_fn(&self) -> syn::ItemFn {
        let name = self.name();
        parse_quote! {
            pub fn #(#name)() {

            }
        }
    }
}
