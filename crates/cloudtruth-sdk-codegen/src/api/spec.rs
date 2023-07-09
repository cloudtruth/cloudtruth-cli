use color_eyre::Result;
use openapiv3::OpenAPI;

use crate::api::ApiOperation;

#[derive(Debug, Clone)]
pub struct ApiSpec {
    operations: Vec<ApiOperation>,
}

impl ApiSpec {
    pub fn new(open_api: OpenAPI) -> Result<Self> {
        let mut operations = open_api
            .operations()
            .filter(|(path, _, _)| path.starts_with("/api/v1/integrations/azure/key_vault/"))
            .map(|(path, method, op)| ApiOperation::from_openapi(path, method, op.clone()))
            .collect::<Result<Vec<ApiOperation>>>()?;
        operations.sort_by(|a, b| a.uri().cmp(b.uri()));
        Ok(Self { operations })
    }

    pub fn operations(&self) -> &[ApiOperation] {
        &self.operations
    }
}
