use crate::database::{auth_details, response_message, ApiError, OpenApiConfig};
use cloudtruth_restapi::apis::api_api::api_schema_retrieve;
use cloudtruth_restapi::apis::Error::ResponseError;
use serde_json::Value;
use std::collections::HashMap;

const REQUIRED_FORMAT: &str = "json";

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

    fn get_schema_map(&self, rest_cfg: &OpenApiConfig) -> Result<HashMap<String, Value>, ApiError> {
        // NOTES:
        // 1. the API fails with YAML format, so always fetch in JSON
        // 2. the language seems to do nothing, even when a bogus value is set
        let response = api_schema_retrieve(rest_cfg, Some(REQUIRED_FORMAT), None);
        match response {
            Ok(map) => Ok(map),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ApiError::UnhandledError(e.to_string())),
        }
    }

    fn output_format(
        &self,
        map: &HashMap<String, Value>,
        format: &str,
    ) -> Result<String, ApiError> {
        match format {
            "yaml" => {
                let result = serde_yaml::to_string(&map).unwrap();
                Ok(result)
            }
            "json" => {
                let result = serde_json::to_string_pretty(&map).unwrap();
                Ok(result)
            }
            fmt => Err(ApiError::UnsupportedFormat(fmt.to_string())),
        }
    }

    pub fn get_schema(&self, rest_cfg: &OpenApiConfig, format: &str) -> Result<String, ApiError> {
        let map = self.get_schema_map(rest_cfg)?;
        self.output_format(&map, format)
    }

    fn version_from_map(&self, map: &HashMap<String, Value>) -> Result<String, ApiError> {
        let info = map.get("info").unwrap();
        let version = info.get("version").unwrap();
        let result = version.as_str().unwrap();
        Ok(result.to_string())
    }

    pub fn get_schema_version(&self, rest_cfg: &OpenApiConfig) -> Result<String, ApiError> {
        let map = self.get_schema_map(rest_cfg)?;
        self.version_from_map(&map)
    }

    fn get_local_schema_map(&self) -> Result<HashMap<String, Value>, ApiError> {
        let file_text = include_str!("../../openapi.yml");
        let result: HashMap<String, Value> = serde_yaml::from_str(file_text)?;
        Ok(result)
    }

    pub fn get_local_schema(&self, format: &str) -> Result<String, ApiError> {
        let map = self.get_local_schema_map()?;
        self.output_format(&map, format)
    }

    pub fn get_local_schema_version(&self) -> Result<String, ApiError> {
        let map = self.get_local_schema_map()?;
        self.version_from_map(&map)
    }
}
