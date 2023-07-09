mod api_method;
mod child_constructor;
mod root_constructors;

pub use api_method::SdkApiMethod;
pub use child_constructor::SdkChildConstructor;
pub use root_constructors::{SdkRootConstructor, SdkStaticRootConstructor};

use super::SdkObject;
use crate::api::ApiOperation;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;

pub trait SdkMethod {
    fn generate_fn(&self) -> syn::ItemFn;
}

impl ToTokens for dyn SdkMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_fn().to_token_stream())
    }
}
