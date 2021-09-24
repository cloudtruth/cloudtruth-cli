use crate::config::Config as CloudTruthConfig;

use cloudtruth_restapi::apis::configuration::{ApiKey, Configuration};
use std::env;

pub type OpenApiConfig = Configuration;

/// This is our fixed page size. The current CLI is not setup to handle paging.
pub const PAGE_SIZE: Option<i32> = None;

/// This is a placeholder for secret wrapping.
pub const WRAP_SECRETS: Option<bool> = None;

fn unquote(orig: &str) -> String {
    orig.trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

/// Extracts the "detail" from the content string, where the content string is a JSON object
/// that contains a "detail" field string value.
pub fn extract_details(content: &str) -> String {
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    let info = match json_result {
        Ok(data) => match data.get("detail") {
            Some(value) => value.to_string(),
            _ => "No details provided.".to_owned(),
        },
        _ => "Did not find details.".to_owned(),
    };
    unquote(&info)
}

pub fn extract_from_json(value: &serde_json::Value) -> String {
    if value.is_string() {
        return value.as_str().unwrap().to_string();
    }

    if let Some(arr) = value.as_array() {
        let mut result = "".to_string();
        for v in arr {
            if !result.is_empty() {
                result.push_str("; ")
            }
            // recursively call into this until we have a string
            let obj_str = extract_from_json(v);
            result.push_str(obj_str.as_str());
        }
        return result;
    }

    if let Some(obj) = value.as_object() {
        if let Some(detail) = obj.get("detail") {
            // recursively get the data out of the string
            return extract_from_json(detail);
        }
    }

    value.to_string()
}

/// Extracts a single string from the content without paying attention to dictionary structure.
pub fn extract_message(content: &str) -> String {
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    match json_result {
        Ok(value) => extract_from_json(&value),
        _ => "No details available".to_string(),
    }
}

/// This extracts information from the content
pub fn generic_response_message(status: &reqwest::StatusCode, content: &str) -> String {
    format!(
        "{} ({}): {}",
        status.canonical_reason().unwrap_or("Unknown Reason"),
        status.as_u16(),
        extract_message(content)
    )
}

fn user_agent_name() -> String {
    format!(
        "{}/{}/{}",
        option_env!("CARGO_PKG_NAME")
            .unwrap_or("cloudtruth")
            .to_string(),
        // TODO: should this be the OpenApi version? or is the CLI version correct?
        option_env!("CARGO_PKG_VERSION")
            .unwrap_or("99.99.99")
            .to_string(),
        env::consts::OS,
    )
}

impl From<&CloudTruthConfig> for OpenApiConfig {
    fn from(ct_cfg: &CloudTruthConfig) -> Self {
        // having a trailing slash confuses the API, so remove any trailing slashes
        let server_url = ct_cfg.server_url.trim_end_matches('/').to_string();
        OpenApiConfig {
            base_path: server_url,
            user_agent: Some(user_agent_name()),
            client: reqwest::Client::builder()
                .timeout(ct_cfg.request_timeout.unwrap())
                .cookie_store(true)
                .build()
                .unwrap(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: Some(ApiKey {
                prefix: Some("Api-Key".to_owned()),
                key: ct_cfg.api_key.clone(),
            }),
            rest_debug: ct_cfg.rest_debug,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;

    #[test]
    #[serial]
    fn conversion_to_openapi() {
        let api_key = "abc123";
        let url = "https://bogushost.com/sna/foo/";
        let ct_cfg = CloudTruthConfig {
            api_key: api_key.to_string(),
            environment: Some("my-env".to_string()),
            profile_name: "prof-name".to_string(),
            project: Some("my-proj".to_string()),
            server_url: url.to_string(),
            request_timeout: Some(Duration::new(120, 0)),
            rest_debug: true,
        };
        let openapi_cfg = OpenApiConfig::from(&ct_cfg);
        // check that the trailing slash removed from the URL
        assert_eq!(
            openapi_cfg.base_path,
            "https://bogushost.com/sna/foo".to_string()
        );
        assert_eq!(openapi_cfg.api_key.unwrap().key, api_key.to_string());
        assert_eq!(openapi_cfg.user_agent.unwrap(), user_agent_name());
        assert_eq!(openapi_cfg.bearer_access_token, None);
        assert_eq!(openapi_cfg.rest_debug, true);
        // unfortunately, no means to interrogate the client to find the timeout
    }

    #[test]
    fn extract_details_test() {
        let content = "";
        let expected = "Did not find details.";
        assert_eq!(expected.to_string(), extract_details(content));

        let content = "{\"speicla\":\"Integration for `github://foo/bar` could not be found.\"}";
        let expected = "No details provided.";
        assert_eq!(expected.to_string(), extract_details(content));

        let content = "{\"detail\":\"Integration for `github://foo/bar` could not be found.\"}";
        let expected = "Integration for `github://foo/bar` could not be found.";
        assert_eq!(expected.to_string(), extract_details(content));
    }
}
