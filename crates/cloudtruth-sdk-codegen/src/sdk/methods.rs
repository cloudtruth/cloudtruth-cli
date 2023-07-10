mod api_method;
mod child_constructor;
mod root_constructors;

pub use api_method::SdkApiMethod;
pub use child_constructor::SdkChildConstructor;
use dyn_clone::DynClone;
pub use root_constructors::{SdkRootConstructor, SdkStaticRootConstructor};

use proc_macro2::TokenStream;
use quote::ToTokens;

pub trait SdkMethod: DynClone {
    fn generate_fn(&self) -> syn::ItemFn;
}

dyn_clone::clone_trait_object!(SdkMethod);

impl ToTokens for dyn SdkMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_fn().to_token_stream())
    }
}
