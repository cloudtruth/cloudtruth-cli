use crate::database::{response_message, ImportDetails, ImportError, OpenApiConfig};
use cloudtruth_restapi::apis::import_api::import_create;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::ImportCreateRequest;

pub struct Imports {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> ImportError {
    ImportError::ResponseError(response_message(status, content))
}

impl Imports {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn import_parameters(
        &self,
        rest_cfg: &OpenApiConfig,
        project_name: &str,
        environment_name: Option<&str>,
        text: &str,
        secrets: &[&str],
        ignore: &[&str],
        inherit_duplicates: bool,
        preview: bool,
        mask_secrets: bool,
    ) -> Result<Vec<ImportDetails>, ImportError> {
        let create_body = ImportCreateRequest {
            project: project_name.to_string(),
            environment: environment_name.map(String::from),
            body: text.to_string(),
            secrets: secrets.iter().map(|&x| String::from(x)).collect(),
            ignore: ignore.iter().map(|&x| String::from(x)).collect(),
            add_project: Some(true),
            add_environment: Some(true),
            add_parameters: Some(true),
            add_overrides: Some(true),
            inherit_on_same: Some(inherit_duplicates),
        };
        let response = import_create(rest_cfg, create_body, Some(mask_secrets), Some(preview));
        match response {
            Ok(api) => Ok(api.parameter.iter().map(ImportDetails::from).collect()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ImportError::UnhandledError(e.to_string())),
        }
    }
}
