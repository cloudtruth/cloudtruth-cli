use crate::config::Config;
use color_eyre::eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigFile {
    profiles: HashMap<String, Profile>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Profile {
    pub api_key: Option<String>,
    pub server_url: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_key: Some("".to_string()),
            server_url: Some("https://ctcaas-graph.cloudtruth.com/graphql".to_string()),
        }
    }
}

impl ConfigFile {
    pub(crate) fn load_profile(config: &str) -> Result<Option<Profile>> {
        let config_map: ConfigFile = serde_yaml::from_str(&config)?;
        let profile = config_map.profiles.get("default");

        Ok(profile.cloned())
    }
}

impl Profile {
    pub(crate) fn load_env_overrides(&mut self) {
        let api_key = env::var("CT_API_KEY");
        if let Ok(api_key) = api_key {
            self.api_key = Some(api_key);
        }

        let server_url = env::var("CT_SERVER_URL");
        if let Ok(server_url) = server_url {
            self.server_url = Some(server_url);
        }
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        if let Some(api_key) = &other.api_key {
            self.api_key = Some(api_key.clone());
        }

        if let Some(server_url) = &other.server_url {
            self.server_url = Some(server_url.clone());
        }
    }

    pub(crate) fn to_config(&self) -> Config {
        Config {
            api_key: self.api_key.as_ref().expect("api_key is empty").clone(),
            server_url: self
                .server_url
                .as_ref()
                .expect("server_url is empty")
                .clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::profiles::ConfigFile;
    use indoc::indoc;

    #[test]
    fn get_api_key_from_profile() {
        let config = indoc!(
            r#"
        profiles:
          default:
            api_key: new_key
        "#
        );

        let profile = ConfigFile::load_profile(config).unwrap();
        assert_eq!(Some("new_key".to_string()), profile.unwrap().api_key)
    }

    #[test]
    fn get_server_url_from_profile() {
        let config = indoc!(
            r#"
        profiles:
          default:
            server_url: http://localhost:7001/graphql
        "#
        );

        let profile = ConfigFile::load_profile(config).unwrap();
        assert_eq!(
            Some("http://localhost:7001/graphql".to_string()),
            profile.unwrap().server_url
        )
    }
}
