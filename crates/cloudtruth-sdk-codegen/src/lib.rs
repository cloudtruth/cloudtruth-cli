pub mod api;
pub mod generator;
pub mod sdk;

use api::ApiSpec;
use color_eyre::Result;
use openapiv3::OpenAPI;

pub fn generate_sdk() -> Result<()> {
    let data = include_str!("../../../openapi.json");
    let open_api: OpenAPI = serde_json::from_str(data).unwrap();
    let spec = ApiSpec::new(open_api)?;
    for op in spec.operations() {
        println!("{}", op.uri());
    }
    Ok(())
}
