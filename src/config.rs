use crate::cli::binary_name;
use color_eyre::eyre::Result;
use config::{ConfigError, File};
use directories::ProjectDirs;
use indoc::{formatdoc, indoc};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

static INSTANCE: OnceCell<Config> = OnceCell::new();

const CONFIG_FILE_NAME: &str = "cli.yml";

#[derive(Serialize, Deserialize, Debug, Default)]
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
        if let Some(project_dirs) = ProjectDirs::from("com", "cloudtruth", "CloudTruth") {
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

    pub fn load_config(jwt: Option<&str>) -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();

        // Set default configuration values.
        settings.set_default("api_key", "")?;
        settings.set_default("server_url", "https://ctcaas-graph.cloudtruth.com/graphql")?;

        // Load settings from the configuration file if it exists.
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                settings.merge(File::from(config_file))?;
            }
        }

        // Load values out of environment variables after loading them out of any config file so
        // that the environment values can take precedence.
        settings.merge(config::Environment::with_prefix("CT"))?;

        // Any arguments supplied via CLI options take precedence over values from both the
        // configuration file as well as the environment.
        if let Some(api_key) = jwt {
            settings.set("api_key", api_key)?;
        }

        Ok(settings.try_into()?)
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
}
