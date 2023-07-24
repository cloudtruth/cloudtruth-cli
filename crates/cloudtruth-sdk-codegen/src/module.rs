use std::{borrow::Cow, fs::File, io::Write, path::Path, process::Command};

use color_eyre::{eyre::Context, Result};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct SdkModule {
    path: Cow<'static, Path>,
    tokens: TokenStream,
}

impl SdkModule {
    pub fn new(path: impl Into<Cow<'static, Path>>, tokens: impl Into<TokenStream>) -> Self {
        SdkModule {
            path: path.into(),
            tokens: tokens.into(),
        }
    }

    fn imports(&self) -> TokenStream {
        quote! {
            use std::sync::Arc;
            use once_cell::sync::OnceCell;
            use reqwest::blocking::Client;
        }
    }

    pub fn write(&self) -> Result<()> {
        let imports = self.imports();
        let tokens = &self.tokens;
        let output = quote! {
            #imports
            #tokens
        };
        File::create(self.path.as_ref())
            .with_context(move || format!("Could not open: {}", self.path.display()))?
            .write_all(output.to_string().as_bytes())?;
        Command::new("rustfmt")
            .arg(self.path.as_os_str())
            .spawn()?
            .wait()?;
        Ok(())
    }
}
