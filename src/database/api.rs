use crate::database::{auth_details, response_message, ApiError, OpenApiConfig};
use cloudtruth_restapi::apis::api_api::api_schema_retrieve;
use cloudtruth_restapi::apis::Error::ResponseError;
use serde_json::Value;
use std::collections::HashMap;

fn response_error(status: &reqwest::StatusCode, content: &str) -> ApiError {
    match status.as_u16() {
        401 => ApiError::Authentication(auth_details(content)),
        403 => ApiError::Authentication(auth_details(content)),
        _ => ApiError::ResponseError(response_message(status, content)),
    }
}

pub struct Api {}

impl Api {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_schema(&self, rest_cfg: &OpenApiConfig, format: &str) -> Result<String, ApiError> {
        // NOTE: the language seems to do nothing, even when a bogus value is set
        let response = api_schema_retrieve(rest_cfg, Some(format), None);
        match response {
            Ok(map) => Ok(map),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ApiError::UnhandledError(e.to_string())),
        }
    }

    fn version_from_map(&self, map: &HashMap<String, Value>) -> Result<String, ApiError> {
        let info = map.get("info").unwrap();
        let version = info.get("version").unwrap();
        let result = version.as_str().unwrap();
        Ok(result.to_string())
    }

    pub fn get_schema_version(&self, rest_cfg: &OpenApiConfig) -> Result<String, ApiError> {
        let data = self.get_schema(rest_cfg, "json")?;
        let map = serde_json::from_str(&data)?;
        self.version_from_map(&map)
    }

    pub fn get_local_schema(&self, format: &str) -> Result<String, ApiError> {
        match format {
            "yaml" => Ok(include_str!("../../openapi.yml").to_string()),
            "json" => Ok(include_str!("../../openapi.json").to_string()),
            _ => Err(ApiError::UnsupportedFormat(format.to_string())),
        }
    }

    pub fn get_local_schema_version(&self) -> Result<String, ApiError> {
        let data = self.get_local_schema("json")?;
        let map = serde_json::from_str(&data)?;
        self.version_from_map(&map)
    }
}
