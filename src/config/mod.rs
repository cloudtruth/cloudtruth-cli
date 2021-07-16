pub(crate) mod env;
mod file;
mod profiles;

use crate::cli::binary_name;
use crate::config::env::ConfigEnv;
use crate::config::file::ConfigFile;
use crate::config::profiles::{Profile, ProfileDetails};
use color_eyre::eyre::Result;
use directories::ProjectDirs;
use indoc::formatdoc;
use once_cell::sync::OnceCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

static INSTANCE: OnceCell<Config> = OnceCell::new();

/// Configuration file name -- location will vary on different systems
const CONFIG_FILE_NAME: &str = "cli.yml";

/// Default OpenApi server URL
pub const DEFAULT_SERVER_URL: &str = "https://api.cloudtruth.io";

/// Default OpenApi server request timeout
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 30;

/// Default environment name.
pub const DEFAULT_ENV_NAME: &str = "default";

/// Default profile name.
pub const DEFAULT_PROF_NAME: &str = "default";

/*************************************************************************
 Environment variables.
 All should start with (CLOUDTRUTH_xxx), with the exception of the old API key.
************************************************************************/
/// Environment variable name used to specify the CloudTruth API value, so it does not need to be
/// specified on the command line.
pub const CT_API_KEY: &str = "CLOUDTRUTH_API_KEY";

/// The old environment variable was named 'CT_API_KEY', and we want to provide a better
/// update process.
pub const CT_OLD_API_KEY: &str = "CT_API_KEY";

/// Environment variable name used to override the default server URL.
pub const CT_SERVER_URL: &str = "CLOUDTRUTH_SERVER_URL";

/// Environment variable name used to override the default server URL.
pub const CT_REQ_TIMEOUT: &str = "CLOUDTRUTH_REQUEST_TIMEOUT";

/// Environment variable name used to set the environment name.
pub const CT_ENVIRONMENT: &str = "CLOUDTRUTH_ENVIRONMENT";

/// Environment variable name used to set the project name.
pub const CT_PROJECT: &str = "CLOUDTRUTH_PROJECT";

/// Environment variable name used to set the profile name.
pub const CT_PROFILE: &str = "CLOUDTRUTH_PROFILE";

/// List of variables to remove to make a clean environment.
#[allow(dead_code)]
pub const CT_APP_REMOVABLE_VARS: &[&str] = &[CT_SERVER_URL, CT_API_KEY, CT_OLD_API_KEY];

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

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub environment: Option<String>,
    pub project: Option<String>,
    pub server_url: String,
    pub request_timeout: Option<Duration>,
}

pub struct ValidationError {
    pub message: String,
    pub help_message: String,
}

pub type ValidationWarning = String;

pub struct ValidationIssues {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl From<Profile> for Config {
    fn from(profile: Profile) -> Self {
        Config {
            api_key: profile.api_key.unwrap_or_else(|| "".to_string()),
            environment: profile.environment,
            project: profile.project,
            server_url: profile
                .server_url
                .unwrap_or_else(|| DEFAULT_SERVER_URL.to_string()),
            request_timeout: Some(Duration::new(
                profile.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
                0,
            )),
        }
    }
}

impl Config {
    pub fn config_file() -> Option<PathBuf> {
        // Load settings from the configuration file if it exists.
        ProjectDirs::from("com", ORGANIZATION_NAME, APPLICATION_NAME)
            .map(|project_dirs| project_dirs.config_dir().join(CONFIG_FILE_NAME))
    }

    fn create_config() -> Result<()> {
        if let Some(config_file) = Self::config_file() {
            if !config_file.exists() {
                if let Some(parent) = config_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                fs::write(config_file, ConfigFile::config_file_template())?;
            }
        }

        Ok(())
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

    pub fn load_config(
        api_key: Option<&str>,
        profile_name: Option<&str>,
        env_name: Option<&str>,
        proj_name: Option<&str>,
    ) -> Result<Self> {
        let mut profile = Profile::default();

        // Load settings from the configuration file if it exists.
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                let config = Self::read_config(config_file.as_path())?;
                let loaded_profile =
                    ConfigFile::load_profile(&config, profile_name.unwrap_or(DEFAULT_PROF_NAME))?;

                profile = profile.merge(&loaded_profile);
            }
        }

        // Load values out of environment variables after loading them out of any config file so
        // that the environment values can take precedence.
        profile = profile.merge(&ConfigEnv::load_profile());

        // Any arguments supplied via CLI options take precedence over values from both the
        // configuration file as well as the environment.
        if let Some(api_key) = api_key {
            profile.api_key = Some(api_key.to_string());
        }
        if let Some(env_name) = env_name {
            profile.environment = Some(env_name.to_string());
        }
        if let Some(proj_name) = proj_name {
            profile.project = Some(proj_name.to_string());
        }

        Ok(profile.into())
    }

    pub fn get_profile_details() -> Result<Vec<ProfileDetails>> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                let config = Self::read_config(config_file.as_path())?;
                profiles = ConfigFile::get_profile_details(&config)?;
            }
        }
        Ok(profiles)
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

    pub fn validate(&self) -> Option<ValidationIssues> {
        let mut errors = vec![];
        let mut warnings = vec![];

        if self.api_key.is_empty() {
            errors.push(ValidationError {
                message: "The API key is missing.".to_string(),
                help_message: formatdoc!(
                    r#"
                        Please either set the `api_key` setting in the configuration file
                        (e.g., run "{} config edit"), pass it as the `--api-key` flag, or
                        supply the API key via the `{}` environment variable."#,
                    CT_API_KEY,
                    binary_name()
                ),
            });
        }

        if ConfigEnv::get_override(CT_OLD_API_KEY).is_some()
            && ConfigEnv::get_override(CT_API_KEY).is_none()
        {
            warnings.push(format!(
                "Please use {} instead of {} to set the API key.",
                CT_API_KEY, CT_OLD_API_KEY
            ));
        }

        if errors.is_empty() && warnings.is_empty() {
            None
        } else {
            Some(ValidationIssues { errors, warnings })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, CT_API_KEY, CT_OLD_API_KEY, CT_SERVER_URL, DEFAULT_PROF_NAME};
    use serial_test::serial;
    use std::env;
    use std::path::PathBuf;

    // Any tests that manipulate environment variables should be run serially as environment
    // variables are a shared global resource. Any such tests should also restore the environment
    // to its pre-test state.

    #[test]
    #[serial]
    fn get_api_key_from_env() {
        env::set_var(CT_API_KEY, "new_key");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.api_key, "new_key");

        env::remove_var(CT_API_KEY);
    }

    #[test]
    #[serial]
    fn get_api_key_from_new_env() {
        env::set_var(CT_OLD_API_KEY, "old_key");
        env::set_var(CT_API_KEY, "new_key");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.api_key, "new_key");

        env::remove_var(CT_API_KEY);
        env::remove_var(CT_OLD_API_KEY);
    }

    #[test]
    #[serial]
    fn api_key_from_args_takes_precedent() {
        env::set_var(CT_API_KEY, "key_from_env");
        let config =
            Config::load_config(Some("key_from_args"), Some(DEFAULT_PROF_NAME), None, None)
                .unwrap();

        assert_eq!(config.api_key, "key_from_args");

        env::remove_var(CT_API_KEY)
    }

    #[test]
    #[serial]
    fn get_server_url_from_env() {
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.server_url, "http://localhost:7001/graphql");

        env::remove_var(CT_SERVER_URL);
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
