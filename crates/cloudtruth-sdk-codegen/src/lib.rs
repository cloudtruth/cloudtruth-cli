pub mod api;
pub mod generator;
pub mod sdk;

use api::ApiSpec;
use color_eyre::Result;
use generator::SdkGenerator;
use openapiv3::OpenAPI;

pub fn generate_sdk() -> Result<()> {
    let data = include_str!("../../../openapi.json");
    let open_api: OpenAPI = serde_json::from_str(data).unwrap();
    let spec = ApiSpec::new(open_api)?;
    let mut generator = SdkGenerator::new(spec);
    generator.root_prefix("/api/v1");
    let _root = generator.build_objects();
    Ok(())
}
