use syn::parse_quote;

use crate::sdk::SdkObject;

use super::SdkMethod;

pub struct SdkRootConstructor<'a>(&'a SdkObject<'a>);

impl<'a> SdkRootConstructor<'a> {
    pub fn new(root: &'a SdkObject<'a>) -> Self {
        SdkRootConstructor(root)
    }
}

impl<'a> SdkMethod for SdkRootConstructor<'a> {
    fn generate_fn(&self) -> syn::ItemFn {
        let name = self.0.name();
        parse_quote! {
            fn new() -> Self {
                #name {
                    client: Arc::new(Client::new()),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SdkStaticRootConstructor();

impl SdkStaticRootConstructor {
    pub fn new() -> Self {
        SdkStaticRootConstructor()
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
