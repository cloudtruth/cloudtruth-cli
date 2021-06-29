use crate::config::Config as CloudTruthConfig;

use cloudtruth_restapi::apis::configuration::{ApiKey, Configuration};

pub type OpenApiConfig = Configuration;

fn user_agent_name() -> String {
    format!(
        "{}/{}",
        option_env!("CARGO_PKG_NAME")
            .unwrap_or("cloudtruth")
            .to_string(),
        // TODO: should this be the OpenApi version? or is the CLI version correct?
        option_env!("CARGO_PKG_VERSION")
            .unwrap_or("99.99.99")
            .to_string(),
    )
}

impl From<&CloudTruthConfig> for OpenApiConfig {
    fn from(ct_cfg: &CloudTruthConfig) -> Self {
        // TODO: use reqwest builder to create client with a timeout?
        let mut result = OpenApiConfig::new();
        result.base_path = ct_cfg.server_url.clone();
        result.user_agent = Some(user_agent_name());
        result.api_key = Some(ApiKey {
            prefix: None,
            key: ct_cfg.api_key.clone(),
        });
        result
    }
}

#[allow(dead_code)]
/// Converts the global CloudTruth config (`config::Config`) into a REST API config
/// (`cloudtruth_restapi::apis::configuration::Configuration`).
pub fn open_api_config() -> OpenApiConfig {
    OpenApiConfig::from(CloudTruthConfig::global())
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
        let url = "https://bogushost.com";
        let ct_cfg = CloudTruthConfig {
            api_key: api_key.to_string(),
            environment: Some("my-env".to_string()),
            project: Some("my-proj".to_string()),
            server_url: url.to_string(),
            request_timeout: Some(Duration::new(30, 0)),
        };
        let openapi_cfg = OpenApiConfig::from(&ct_cfg);
        assert_eq!(openapi_cfg.base_path, url.to_string());
        assert_eq!(openapi_cfg.api_key.unwrap().key, api_key.to_string());
        assert_eq!(openapi_cfg.user_agent.unwrap(), user_agent_name());
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
        assert_eq!(openapi_cfg.api_key.unwrap().key, api_key.to_string());
        assert_eq!(openapi_cfg.user_agent.unwrap(), user_agent_name());

        env::remove_var(CT_API_KEY);
        env::remove_var(CT_SERVER_URL);
    }
}
