use crate::config::profiles::Profile;
use crate::config::{CT_API_KEY, CT_ENVIRONMENT, CT_OLD_API_KEY, CT_PROJECT, CT_SERVER_URL};
use std::env;

pub struct ConfigEnv {}

impl ConfigEnv {
    pub(crate) fn load_profile() -> Profile {
        Profile {
            api_key: ConfigEnv::get_override(CT_API_KEY)
                .or_else(|| ConfigEnv::get_override(CT_OLD_API_KEY)),
            description: None,
            environment: ConfigEnv::get_override(CT_ENVIRONMENT),
            project: ConfigEnv::get_override(CT_PROJECT),
            server_url: ConfigEnv::get_override(CT_SERVER_URL),
            source_profile: None,
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
}

#[cfg(test)]
mod tests {
    use crate::config::profiles::Profile;
    use crate::config::{
        ConfigEnv, CT_API_KEY, CT_ENVIRONMENT, CT_OLD_API_KEY, CT_PROJECT, CT_SERVER_URL,
    };
    use serial_test::serial;
    use std::env;

    fn remove_env_vars() {
        env::remove_var(CT_API_KEY);
        env::remove_var(CT_ENVIRONMENT);
        env::remove_var(CT_OLD_API_KEY);
        env::remove_var(CT_PROJECT);
        env::remove_var(CT_SERVER_URL);
    }

    #[test]
    #[serial]
    fn create_profile_from_empty_env() {
        remove_env_vars();
        assert_eq!(
            Profile {
                api_key: None,
                description: None,
                environment: None,
                project: None,
                server_url: None,
                source_profile: None
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
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");

        assert_eq!(
            Profile {
                api_key: Some("new_key".to_string()),
                description: None,
                environment: Some("my_environment".to_string()),
                project: Some("skunkworks".to_string()),
                server_url: Some("http://localhost:7001/graphql".to_string()),
                source_profile: None
            },
            ConfigEnv::load_profile()
        );
    }

    #[test]
    #[serial]
    fn create_profile_new_env_takes_precedence() {
        remove_env_vars();
        env::set_var(CT_API_KEY, "new_key");
        env::set_var(CT_ENVIRONMENT, "my_env");
        env::set_var(CT_OLD_API_KEY, "old_key");
        env::set_var(CT_PROJECT, "skunkworks");
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");

        assert_eq!(
            Profile {
                api_key: Some("new_key".to_string()),
                description: None,
                environment: Some("my_env".to_string()),
                project: Some("skunkworks".to_string()),
                server_url: Some("http://localhost:7001/graphql".to_string()),
                source_profile: None
            },
            ConfigEnv::load_profile()
        );
    }

    #[test]
    #[serial]
    fn create_profile_from_old_env() {
        remove_env_vars();
        env::set_var(CT_ENVIRONMENT, "my_environ");
        env::set_var(CT_PROJECT, "skunkworks");
        env::set_var(CT_OLD_API_KEY, "old_key");
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");

        assert_eq!(
            Profile {
                api_key: Some("old_key".to_string()),
                description: None,
                environment: Some("my_environ".to_string()),
                project: Some("skunkworks".to_string()),
                server_url: Some("http://localhost:7001/graphql".to_string()),
                source_profile: None
            },
            ConfigEnv::load_profile()
        );
    }
}
