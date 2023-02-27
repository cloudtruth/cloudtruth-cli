use crate::config::profiles::Profile;
use crate::config::{
    CT_ACCEPT_INVALID_CERTS, CT_API_KEY, CT_ENVIRONMENT, CT_PROJECT, CT_REQ_TIMEOUT, CT_REST_DEBUG,
    CT_REST_PAGE_SIZE, CT_REST_SUCCESS, CT_SERVER_URL,
};
use crate::utils::warn_user;
use std::env;
use std::str::FromStr;

pub struct ConfigEnv {}

impl ConfigEnv {
    pub(crate) fn load_profile() -> Profile {
        Profile {
            api_key: Self::get_override(CT_API_KEY),
            description: None,
            environment: Self::get_override(CT_ENVIRONMENT),
            project: Self::get_override(CT_PROJECT),
            request_timeout: Self::parse_override(CT_REQ_TIMEOUT),
            rest_debug: Self::get_rest_debug(),
            rest_success: Self::get_rest_success(),
            rest_page_size: Self::parse_override(CT_REST_PAGE_SIZE),
            server_url: Self::get_override(CT_SERVER_URL),
            source_profile: None,
            accept_invalid_certs: Self::get_accept_invalid_certs(),
        }
    }

    pub fn get_override(config_name: &str) -> Option<String> {
        env::var(config_name).ok().map(|val| {
            if val.is_empty() {
                warn_user(format!(
                    "{config_name} is defined but empty. It could be accidentally shadowing profile config."
                ));
            }
            val.trim().to_owned()
        })
    }

    pub fn parse_override<T: FromStr>(config_name: &str) -> Option<T> {
        Self::get_override(config_name).and_then(|config_value| {
            config_value.parse().ok().or_else(|| {
                warn_user(format!("Could not parse {config_name}: {config_value}"));
                None
            })
        })
    }

    pub fn get_rest_page_size() -> Option<i32> {
        Self::parse_override(CT_REST_PAGE_SIZE)
    }

    pub fn get_rest_debug() -> Option<bool> {
        Self::get_override(CT_REST_DEBUG)
            .map(|e| matches!(e.to_lowercase().as_str(), "true" | "1" | "yes"))
    }

    pub fn get_accept_invalid_certs() -> Option<bool> {
        Self::get_override(CT_ACCEPT_INVALID_CERTS)
            .map(|e| matches!(e.to_lowercase().as_str(), "true" | "1" | "yes"))
    }

    pub fn get_rest_success() -> Vec<String> {
        Self::get_override(CT_REST_SUCCESS)
            .iter()
            .flat_map(|env_str| env_str.split(',').map(|i| i.trim().to_owned()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn remove_env_vars() {
        env::remove_var(CT_API_KEY);
        env::remove_var(CT_ENVIRONMENT);
        env::remove_var(CT_PROJECT);
        env::remove_var(CT_REQ_TIMEOUT);
        env::remove_var(CT_SERVER_URL);
        env::remove_var(CT_REST_DEBUG);
        env::remove_var(CT_REST_SUCCESS);
        env::remove_var(CT_REST_PAGE_SIZE);
        env::remove_var(CT_ACCEPT_INVALID_CERTS);
    }

    #[test]
    #[serial]
    fn create_profile_from_empty_env() {
        remove_env_vars();
        assert_eq!(
            Profile {
                ..Profile::default()
            },
            ConfigEnv::load_profile()
        );
    }

    #[test]
    #[serial]
    fn create_profile_from_env() {
        remove_env_vars();
        env::set_var(CT_API_KEY, "new_key");
        env::set_var(CT_ENVIRONMENT, "my_environment");
        env::set_var(CT_PROJECT, "skunkworks");
        env::set_var(CT_REQ_TIMEOUT, "500");
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");
        env::set_var(CT_REST_DEBUG, "true");
        env::set_var(CT_REST_SUCCESS, "sna,foo,bar");
        env::set_var(CT_ACCEPT_INVALID_CERTS, "1");

        assert_eq!(
            Profile {
                api_key: Some("new_key".to_string()),
                description: None,
                environment: Some("my_environment".to_string()),
                project: Some("skunkworks".to_string()),
                request_timeout: Some(500),
                rest_debug: Some(true),
                rest_success: vec!["sna".to_string(), "foo".to_string(), "bar".to_string()],
                rest_page_size: None,
                server_url: Some("http://localhost:7001/graphql".to_string()),
                source_profile: None,
                accept_invalid_certs: Some(true)
            },
            ConfigEnv::load_profile()
        );
    }
}
