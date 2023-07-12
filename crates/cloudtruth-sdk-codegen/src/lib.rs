pub mod api;
pub mod generator;
pub mod module;
mod names;
pub mod sdk;

use api::ApiSpec;
use color_eyre::Result;
use generator::SdkGenerator;
use module::SdkModule;
use openapiv3::OpenAPI;
use quote::quote;

macro_rules! sdk_path {
    ($($path:expr),* $(,)?) => {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../cloudtruth-sdk/", $($path),*))
    };
}

pub(crate) use sdk_path;

/// Shorthand to quickly create an Ident with call_site macro hygiene
/// Since this is a codegen project, macro hygiene isn't very important for us(?)
macro_rules! ident {
    ($path:expr) => {
        proc_macro2::Ident::new(&*($path), proc_macro2::Span::call_site())
    };
}

pub(crate) use ident;

pub fn generate_sdk() -> Result<()> {
    let data = include_str!("../../../openapi.json");
    let open_api: OpenAPI = serde_json::from_str(data).unwrap();
    let spec = ApiSpec::new(open_api)?;
    let mut generator = SdkGenerator::new(spec);
    generator.root_prefix("/api/v1");
    let objects = generator.build_objects();
    SdkModule::new(
        sdk_path!("src/lib.rs"),
        quote! {
            #(#objects )*
        },
    )
    .write()?;
    Ok(())
}
