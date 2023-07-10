use std::borrow::Cow;

use quote::{quote, ToTokens};

use super::SdkObject;

pub struct SdkRoot<'a> {
    type_name: Cow<'a, str>,
    children: Vec<SdkObject<'a>>,
}

impl<'a> SdkRoot<'a> {
    pub fn new(type_name: impl Into<Cow<'a, str>>) -> Self {
        let type_name = type_name.into();
        Self {
            type_name,
            children: Vec::new(),
        }
    }
}

impl<'a> HasChildren<SdkObject<'a>> for SdkRoot<'a> {
    fn children(&self) -> &[SdkObject<'a>] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut [SdkObject<'a>] {
        &mut self.children
    }
    fn add_child(&mut self, child: SdkObject<'a>) {
        self.children.push(child);
    }
}

impl<'a> ToTokens for SdkRoot<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { type_name, .. } = self;
        tokens.extend(quote! {
            use std::sync::Arc;

            use once_cell::sync::OnceCell;
            use reqwest::Client;

            pub struct #type_name {
                pub client: Arc<Client>,
            }

            impl #type_name {
                fn new() -> Self {
                    CloudtruthSdk {
                        client: Arc::new(Client::new()),
                    }
                }
                pub fn instance() -> &'static Self {
                    static ONCE: OnceCell<CloudtruthSdk> = OnceCell::new();
                    ONCE.get_or_init(CloudtruthSdk::new)
                }
            }

        })
    }
}
