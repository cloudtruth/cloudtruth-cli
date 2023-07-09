mod method;
mod modules;

use color_eyre::Result;
use openapiv3::OpenAPI;

use crate::method::SdkOperation;

pub fn generate_sdk() -> Result<()> {
    let data = include_str!("../../../openapi.json");
    let openapi: OpenAPI = serde_json::from_str(data).unwrap();
    let _methods = openapi
        .operations()
        .filter(|(path, _, _)| path.starts_with("/api/v1/integrations/azure/key_vault/"))
        .map(|(path, method, op)| SdkOperation::from_openapi_operation(path, method, op.clone()))
        .collect::<Vec<_>>();
    modules::root().write()?;
    Ok(())
}
