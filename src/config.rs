use crate::cli::binary_name;
use color_eyre::eyre::Result;
use directories::ProjectDirs;
use indoc::{formatdoc, indoc};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

static INSTANCE: OnceCell<Config> = OnceCell::new();

const CONFIG_FILE_NAME: &str = "cli.yml";

// Linux follows the XDG directory layout and creates one directory per application. However, our
// configuration files indicate the application name, so we can use a shared directory.
#[cfg(target_os = "linux")]
const APPLICATION_NAME: &str = "CloudTruth";

#[cfg(not(target_os = "linux"))]
const APPLICATION_NAME: &str = "CloudTruth CLI";

#[cfg(target_os = "macos")]
const ORGANIZATION_NAME: &str = "cloudtruth";

#[cfg(not(target_os = "macos"))]
const ORGANIZATION_NAME: &str = "CloudTruth";

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ConfigMap {
    profiles: HashMap<String, Profile>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
struct Profile {
    api_key: Option<String>,
    server_url: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_key: Some("".to_string()),
            server_url: Some("https://ctcaas-graph.cloudtruth.com/graphql".to_string()),
        }
    }
}

impl Profile {
    fn load_env_overrides(&mut self) {
        let api_key = env::var("CT_API_KEY");
        if let Ok(api_key) = api_key {
            self.api_key = Some(api_key);
        }

        let server_url = env::var("CT_SERVER_URL");
        if let Ok(server_url) = server_url {
            self.server_url = Some(server_url);
        }
    }

    fn merge(&mut self, other: &Self) {
        if let Some(api_key) = &other.api_key {
            self.api_key = Some(api_key.clone());
        }

        if let Some(server_url) = &other.server_url {
            self.server_url = Some(server_url.clone());
        }
    }

    fn to_config(&self) -> Config {
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

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub server_url: String,
}

pub struct ValidationError {
    pub message: String,
    pub help_message: String,
}

impl Config {
    fn config_file() -> Option<PathBuf> {
        // Load settings from the configuration file if it exists.
        if let Some(project_dirs) = ProjectDirs::from("com", ORGANIZATION_NAME, APPLICATION_NAME) {
            Some(project_dirs.config_dir().join(CONFIG_FILE_NAME))
        } else {
            None
        }
    }

    fn create_config() -> Result<()> {
        if let Some(config_file) = Self::config_file() {
            if !config_file.exists() {
                if let Some(parent) = config_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                fs::write(config_file, Self::default_config_file())?;
            }
        }

        Ok(())
    }

    fn default_config_file() -> &'static str {
        indoc!(
            r#"
            # Your CloudTruth API key.
            api_key: ""
        "#
        )
    }

    pub fn global() -> &'static Self {
        INSTANCE.get().expect("configuration is not initialized")
    }

    pub fn init_global(config: Self) {
        INSTANCE.set(config).unwrap()
    }

    fn read_config(config_file: &Path) -> Result<String> {
        let contents = fs::read_to_string(config_file)?;

        Ok(contents)
    }

    pub fn load_config(api_key: Option<&str>) -> Result<Self> {
        let mut profile = Profile::default();

        // Load settings from the configuration file if it exists.
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                let config = Self::read_config(config_file.as_path())?;
                let config_map: ConfigMap = serde_yaml::from_str(&config)?;

                let default_profile = config_map.profiles.get("default");
                if let Some(default_profile) = default_profile {
                    profile.merge(default_profile);
                }
            }
        }

        // Load values out of environment variables after loading them out of any config file so
        // that the environment values can take precedence.
        profile.load_env_overrides();

        // Any arguments supplied via CLI options take precedence over values from both the
        // configuration file as well as the environment.
        if let Some(api_key) = api_key {
            profile.api_key = Some(api_key.to_string());
        }

        Ok(profile.to_config())
    }

    pub fn edit() -> Result<()> {
        if let Some(config_file) = Self::config_file() {
            if !config_file.exists() {
                Self::create_config()?;
            }

            edit::edit_file(config_file)?;
        }

        Ok(())
    }

    pub fn validate(&self) -> Option<Vec<ValidationError>> {
        let mut messages = vec![];

        if self.api_key.is_empty() {
            messages.push(ValidationError {
                message: "The API key is missing.".to_string(),
                help_message: formatdoc!(
                    r#"
                        Please either set the `api_key` setting in the configuration file
                        (e.g., run "{} config edit"), pass it as the `--api-key` flag, or
                        supply the API key via the `CT_API_KEY` environment variable."#,
                    binary_name()
                ),
            });
        }

        if messages.is_empty() {
            None
        } else {
            Some(messages)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use serial_test::serial;
    use std::env;
    use std::path::PathBuf;

    // Any tests that manipulate environment variables should be run serially as environment
    // variables are a shared global resource. Any such tests should also restore the environment
    // to its pre-test state.

    #[test]
    #[serial]
    fn get_api_key_from_env() {
        env::set_var("CT_API_KEY", "new_key");
        let config = Config::load_config(None).unwrap();

        assert_eq!(config.api_key, "new_key");

        env::remove_var("CT_API_KEY");
    }

    #[test]
    #[serial]
    fn api_key_from_args_takes_precedent() {
        env::set_var("CT_API_KEY", "key_from_env");
        let config = Config::load_config(Some("key_from_args")).unwrap();

        assert_eq!(config.api_key, "key_from_args");

        env::remove_var("CT_API_KEY")
    }

    #[test]
    #[serial]
    fn get_server_url_from_env() {
        env::set_var("CT_SERVER_URL", "http://localhost:7001/graphql");
        let config = Config::load_config(None).unwrap();

        assert_eq!(config.server_url, "http://localhost:7001/graphql");

        env::remove_var("CT_SERVER_URL");
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[serial]
    fn get_config_file_location() {
        let user_dirs = directories::UserDirs::new().unwrap();
        let home_dir = user_dirs.home_dir();
        let mut expected = PathBuf::new();
        expected.push(home_dir);
        expected.push(".config/cloudtruth/cli.yml");

        assert_eq!(Config::config_file(), Some(expected))
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[serial]
    fn get_config_file_location_with_custom_xdg_config() {
        let old_value = env::var("XDG_CONFIG_HOME ");
        env::set_var("XDG_CONFIG_HOME", "/tmp");

        let expected = PathBuf::from("/tmp/cloudtruth/cli.yml");

        assert_eq!(Config::config_file(), Some(expected));

        if let Ok(old_value) = old_value {
            env::set_var("XDG_CONFIG_HOME", old_value);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn get_config_file_location() {
        let user_dirs = directories::UserDirs::new().unwrap();
        let home_dir = user_dirs.home_dir();
        let mut expected = PathBuf::new();
        expected.push(home_dir);
        expected.push("Library/Application Support/com.cloudtruth.CloudTruth-CLI/cli.yml");

        assert_eq!(Config::config_file(), Some(expected))
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn get_config_file_location() {
        let user_dirs = directories::UserDirs::new().unwrap();
        let home_dir = user_dirs.home_dir();
        let mut expected = PathBuf::new();
        expected.push(home_dir);
        expected.push("AppData/Roaming/CloudTruth/CloudTruth CLI/config/cli.yml");

        assert_eq!(Config::config_file(), Some(expected))
    }
}
