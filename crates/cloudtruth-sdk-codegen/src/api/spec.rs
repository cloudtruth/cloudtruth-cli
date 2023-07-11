use std::rc::Rc;

use color_eyre::Result;
use openapiv3::OpenAPI;

use crate::api::ApiOperation;

#[derive(Debug, Clone)]
pub struct ApiSpec {
    operations: Vec<Rc<ApiOperation>>,
}

impl ApiSpec {
    pub fn new(open_api: OpenAPI) -> Result<Self> {
        let mut operations = open_api
            .operations()
            .filter(|(path, _, _)| path.starts_with("/api/v1/integrations/azure/key_vault/"))
            .map(|(path, method, op)| {
                Ok(Rc::new(ApiOperation::from_openapi(
                    path,
                    method,
                    op.clone(),
                )?))
            })
            .collect::<Result<Vec<Rc<ApiOperation>>>>()?;
        operations.sort_by(|a, b| a.uri().cmp(b.uri()));
        Ok(Self { operations })
    }

    pub fn operations(&self) -> &[Rc<ApiOperation>] {
        &self.operations
    }
}
