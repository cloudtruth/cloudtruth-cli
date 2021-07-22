use crate::config::Config as CloudTruthConfig;

use cloudtruth_restapi::apis::configuration::{ApiKey, Configuration};
use once_cell::sync::OnceCell;
use std::env;

pub type OpenApiConfig = Configuration;

static INSTANCE: OnceCell<OpenApiConfig> = OnceCell::new();

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
    info.trim_start_matches('\"')
        .trim_end_matches('\"')
        .to_string()
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
                .build()
                .unwrap(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: Some(ApiKey {
                prefix: Some("Api-Key".to_owned()),
                key: ct_cfg.api_key.clone(),
            }),
            cookie: None,
        }
    }
}

#[allow(dead_code)]
/// Converts the global CloudTruth config (`config::Config`) into a REST API config
/// (`cloudtruth_restapi::apis::configuration::Configuration`).
pub fn open_api_config() -> &'static OpenApiConfig {
    INSTANCE.get_or_init(|| OpenApiConfig::from(CloudTruthConfig::global()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CT_API_KEY, CT_SERVER_URL};
    use serial_test::serial;
    use std::env;
    use std::time::Duration;

    #[test]
    #[serial]
    fn conversion_to_openapi() {
        let api_key = "abc123";
        let url = "https://bogushost.com/sna/foo/";
        let ct_cfg = CloudTruthConfig {
            api_key: api_key.to_string(),
            environment: Some("my-env".to_string()),
            project: Some("my-proj".to_string()),
            server_url: url.to_string(),
            request_timeout: Some(Duration::new(120, 0)),
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
        // unfortunately, no means to interrogate the client to find the timeout
    }

    #[test]
    #[serial]
    fn openapi_config_from_global() {
        let api_key = "my-api-key";
        let url = "https://another-fake-server";
        env::set_var(CT_API_KEY, api_key);
        env::set_var(CT_SERVER_URL, url);

        CloudTruthConfig::init_global(
            CloudTruthConfig::load_config(None, None, None, None).unwrap(),
        );
        let openapi_cfg = open_api_config();
        assert_eq!(openapi_cfg.base_path, url.to_string());
        assert_eq!(
            openapi_cfg.api_key.clone().unwrap().key,
            api_key.to_string()
        );
        assert_eq!(openapi_cfg.user_agent.clone().unwrap(), user_agent_name());
        assert_eq!(openapi_cfg.bearer_access_token, None);

        env::remove_var(CT_API_KEY);
        env::remove_var(CT_SERVER_URL);
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
