use crate::config::profiles::Profile;
use crate::config::{
    CT_ACCEPT_INVALID_CERTS, CT_API_KEY, CT_ENVIRONMENT, CT_PROJECT, CT_REQ_TIMEOUT, CT_REST_DEBUG,
    CT_REST_PAGE_SIZE, CT_REST_SUCCESS, CT_SERVER_URL,
};
use std::env;

pub struct ConfigEnv {}

impl ConfigEnv {
    pub(crate) fn load_profile() -> Profile {
        Profile {
            api_key: Self::get_override(CT_API_KEY),
            description: None,
            environment: Self::get_override(CT_ENVIRONMENT),
            project: Self::get_override(CT_PROJECT),
            request_timeout: Self::get_duration_override(),
            rest_debug: Self::get_rest_debug(),
            rest_success: Self::get_rest_success(),
            rest_page_size: Self::get_rest_page_size(),
            server_url: Self::get_override(CT_SERVER_URL),
            source_profile: None,
            accept_invalid_certs: Self::get_accept_invalid_certs(),
        }
    }

    pub fn get_override(config_name: &str) -> Option<String> {
        let value = env::var(config_name);

        if let Ok(value) = value {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_duration_override() -> Option<u64> {
        let mut result = None;
        if let Some(dur_str) = Self::get_override(CT_REQ_TIMEOUT) {
            if let Ok(dur_val) = dur_str.parse::<u64>() {
                result = Some(dur_val);
            }
        }
        result
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
        let mut result = vec![];
        if let Some(env_str) = Self::get_override(CT_REST_SUCCESS) {
            let items: Vec<&str> = env_str.split(',').collect();
            for i in items {
                result.push(i.trim().to_string());
            }
        }
        result
    }

    pub fn get_rest_page_size() -> Option<i32> {
        match Self::get_override(CT_REST_PAGE_SIZE) {
            Some(env_value) => match env_value.trim().parse() {
                Ok(int_value) => Some(int_value),
                _ => None,
            },
            _ => None,
        }
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
