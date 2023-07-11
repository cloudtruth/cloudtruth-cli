pub mod api;
pub mod generator;
pub mod module;
pub mod sdk;

use api::ApiSpec;
use color_eyre::Result;
use generator::SdkGenerator;
use module::SdkModule;
use openapiv3::OpenAPI;
use quote::quote;

#[macro_export]
macro_rules! sdk_path {
    ($($path:expr),* $(,)?) => {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../cloudtruth-sdk/", $($path),*))
    };
}

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
            use std::sync::Arc;
            #(#objects )*
        },
    )
    .write()?;
    Ok(())
}
