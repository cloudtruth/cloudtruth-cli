use std::rc::Rc;

use syn::parse_quote;

use crate::sdk::SdkObject;

use super::SdkMethod;

#[derive(Clone)]
pub struct SdkRootConstructor {
    name: Rc<str>,
}

impl SdkRootConstructor {
    pub fn new(object: &SdkObject) -> Self {
        let name = object.name().clone();
        SdkRootConstructor { name }
    }
}

impl SdkMethod for SdkRootConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        let name = &self.name;
        parse_quote! {
            fn new() -> Self {
                #name {
                    client: Arc::new(Client::new()),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SdkStaticRootConstructor();

impl SdkStaticRootConstructor {
    pub fn new() -> Self {
        SdkStaticRootConstructor::default()
    }
}

impl SdkMethod for SdkStaticRootConstructor {
    fn generate_fn(&self) -> syn::ItemFn {
        parse_quote! {
            pub fn instance() -> &'static Self {
                static ONCE: OnceCell<CloudtruthSdk> = OnceCell::new();
                ONCE.get_or_init(CloudtruthSdk::new)
            }
        }
    }
}
