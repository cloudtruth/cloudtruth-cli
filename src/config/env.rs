use crate::config::profiles::Profile;
use std::env;

pub(crate) struct ConfigEnv {}

const ENV_VAR_PREFIX: &str = "CT_";

impl ConfigEnv {
    pub(crate) fn load_profile() -> Profile {
        Profile {
            api_key: ConfigEnv::get_override("api_key"),
            server_url: ConfigEnv::get_override("server_url"),
        }
    }

    fn get_override(config_name: &str) -> Option<String> {
        let value = env::var(format!("{}{}", ENV_VAR_PREFIX, config_name.to_uppercase()));

        if let Ok(value) = value {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::env::ConfigEnv;
    use crate::config::profiles::Profile;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn create_profile_from_empty_env() {
        assert_eq!(
            Profile {
                api_key: None,
                server_url: None
            },
            ConfigEnv::load_profile()
        );
    }

    #[test]
    #[serial]
    fn create_profile_from_env() {
        env::set_var("CT_API_KEY", "new_key");
        env::set_var("CT_SERVER_URL", "http://localhost:7001/graphql");

        assert_eq!(
            Profile {
                api_key: Some("new_key".to_string()),
                server_url: Some("http://localhost:7001/graphql".to_string())
            },
            ConfigEnv::load_profile()
        );

        env::remove_var("CT_API_KEY");
        env::remove_var("CT_SERVER_URL");
    }
}
