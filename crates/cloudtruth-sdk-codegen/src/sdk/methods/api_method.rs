use std::rc::Rc;

use syn::parse_quote;

use crate::api::ApiOperation;
use crate::{ident, names};

use super::SdkMethod;

#[derive(Debug, Clone)]
pub struct SdkApiMethod {
    api_op: Rc<ApiOperation>,
    fn_name: Rc<str>,
}

impl SdkApiMethod {
    pub fn new(url_path: impl AsRef<str>, api_op: impl Into<Rc<ApiOperation>>) -> Self {
        let api_op = api_op.into();
        let fn_name = Self::create_method_name(url_path.as_ref(), &api_op);
        SdkApiMethod { api_op, fn_name }
    }

    fn create_method_name(url_path: &str, api_op: &ApiOperation) -> Rc<str> {
        let op_id_parts = api_op.operation_id().unwrap().split('_');
        let path_parts = url_path
            .split(|c| c == '/' || c == '_')
            .map(names::trim_path_var_brackets)
            .collect::<Vec<&str>>();
        op_id_parts
            .filter(|op_id_part| !path_parts.contains(op_id_part))
            .collect::<Vec<&str>>()
            .join("_")
            .into()
    }

    fn fn_name(&self) -> &Rc<str> {
        &self.fn_name
    }
}

impl SdkMethod for SdkApiMethod {
    fn generate_fn(&self) -> syn::ItemFn {
        let name = ident!(self.fn_name());
        parse_quote! {
            pub fn #name() {

            }
        }
    }
}
