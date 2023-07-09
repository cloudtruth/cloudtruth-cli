mod root;

pub use root::root;

use std::{borrow::Cow, fs::File, io::Write, path::Path, process::Command};

use color_eyre::{eyre::Context, Result};
use proc_macro2::TokenStream;

#[macro_export]
macro_rules! sdk_path {
    ($($path:expr),* $(,)?) => {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../cloudtruth-sdk/", $($path),*))
    };
}

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

    pub fn write(&self) -> Result<()> {
        File::create(self.path.as_ref())
            .with_context(move || format!("Could not open: {}", self.path.display()))?
            .write_all(self.tokens.to_string().as_bytes())?;
        Command::new("rustfmt")
            .arg(self.path.as_os_str())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

pub fn root() -> SdkModule {
    let root = SdkRoot::new();
    SdkModule::new(sdk_path!("src/lib.rs"), quote!(#root))
}
