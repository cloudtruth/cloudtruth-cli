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
 Environment variables - all should start with (CLOUDTRUTH_).
************************************************************************/
/// Environment variable name used to specify the CloudTruth API value, so it does not need to be
/// specified on the command line.
pub const CT_API_KEY: &str = "CLOUDTRUTH_API_KEY";

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

/// Environment variable name used to enable REST debugging statements.
pub const CT_REST_DEBUG: &str = "CLOUDTRUTH_REST_DEBUG";

/// List of variables to remove to make a clean environment.
#[allow(dead_code)]
pub const CT_APP_REMOVABLE_VARS: &[&str] = &[CT_SERVER_URL, CT_API_KEY];

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
    pub profile_name: String,
    pub project: Option<String>,
    pub server_url: String,
    pub request_timeout: Option<Duration>,
    pub rest_debug: bool,
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

fn profile_into_config(prof_name: &str, profile: &Profile) -> Config {
    Config {
        api_key: profile.api_key.clone().unwrap_or_default(),
        environment: profile.environment.clone(),
        profile_name: prof_name.to_string(),
        project: profile.project.clone(),
        server_url: profile
            .server_url
            .clone()
            .unwrap_or_else(|| DEFAULT_SERVER_URL.to_string()),
        request_timeout: Some(Duration::new(
            profile.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
            0,
        )),
        rest_debug: profile.rest_debug.unwrap_or(false),
    }
}

const SRC_DEFAULT: &str = "default";
const SRC_ARG: &str = "argument";
const SRC_ENV: &str = "shell";
const SRC_PROFILE: &str = "profile";
const SRC_BINARY: &str = "binary";

const PARAM_PROFILE: &str = "Profile";
const PARAM_API_KEY: &str = "API key";
const PARAM_PROJECT: &str = "Project";
const PARAM_ENVIRONMENT: &str = "Environment";
const PARAM_SERVER_URL: &str = "Server URL";
const PARAM_REQUEST_TIMEOUT: &str = "Request timeout";
const PARAM_REST_DEBUG: &str = "REST debug";
const PARAM_CLI_VERSION: &str = "CLI version";

#[derive(Clone, Debug)]
pub struct ConfigValue {
    pub name: String,
    pub value: String,
    pub source: String,
    pub secret: bool,
    pub extension: bool,
}

impl Config {
    pub fn config_file() -> Option<PathBuf> {
        // Load settings from the configuration file if it exists.
        ProjectDirs::from("com", ORGANIZATION_NAME, APPLICATION_NAME)
            .map(|project_dirs| project_dirs.config_dir().join(CONFIG_FILE_NAME))
    }

    pub fn filename() -> String {
        Config::config_file()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
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
        let prof_name = profile_name.unwrap_or(DEFAULT_PROF_NAME);

        // Load settings from the configuration file
        if let Some(config_file) = Self::config_file() {
            // use the template as the config, if the file does not exist
            let config = if config_file.exists() {
                Self::read_config(config_file.as_path())?
            } else {
                ConfigFile::config_file_template().to_string()
            };
            let loaded_profile = ConfigFile::load_profile(&config, prof_name)?;
            profile = profile.merge(&loaded_profile);
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

        let config = profile_into_config(prof_name, &profile);
        Ok(config)
    }

    pub fn update_profile(
        profile_name: &str,
        api_key: Option<&str>,
        description: Option<&str>,
        environment: Option<&str>,
        project: Option<&str>,
        source: Option<&str>,
    ) -> Result<()> {
        if let Some(filename) = Self::config_file() {
            if !filename.exists() {
                Self::create_config()?;
            }
            let content = Self::read_config(filename.as_path())?;
            let updated = ConfigFile::set_profile(
                &content,
                profile_name,
                api_key,
                description,
                environment,
                project,
                source,
            )?;
            fs::write(filename.as_path(), updated)?;
        }
        Ok(())
    }

    pub fn delete_profile(profile_name: &str) -> Result<()> {
        if let Some(filename) = Self::config_file() {
            let content: String;
            if filename.exists() {
                content = Self::read_config(filename.as_path())?;
            } else {
                content = ConfigFile::config_file_template().to_string();
            }
            let updated = ConfigFile::remove_profile(&content, profile_name)?;
            fs::write(filename.as_path(), updated)?;
        }
        Ok(())
    }

    /// Gets a single profile details (or None) by name
    pub fn get_profile_details_by_name(profile_name: &str) -> Result<Option<ProfileDetails>> {
        let mut details: Option<ProfileDetails> = None;
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                let config = Self::read_config(config_file.as_path())?;
                details = ConfigFile::get_details_by_name(&config, profile_name)?;
            }
        }
        Ok(details)
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

        if self.api_key.is_empty() {
            errors.push(ValidationError {
                message: "The API key is missing.".to_string(),
                help_message: formatdoc!(
                    r#"
                        Please set the `api_key` using one of the following `{}` methods:
                        1. Update the configuration profile `api_key` value
                           using "login", "config edit", or "config profile set" commands,
                        2. Provide an `--api-key` value, or
                        3. Supply the API key via the `{}` environment variable."#,
                    binary_name(),
                    CT_API_KEY,
                ),
            });
        }

        if errors.is_empty() {
            None
        } else {
            Some(ValidationIssues {
                errors,
                warnings: vec![],
            })
        }
    }

    fn profile_details_list_for(profile_name: &str) -> Vec<ProfileDetails> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                if let Ok(config) = Self::read_config(config_file.as_path()) {
                    if let Ok(details) = ConfigFile::get_details_for(&config, profile_name) {
                        profiles = details;
                    }
                }
            }
        }
        profiles
    }

    pub fn get_sources(
        profile_name: Option<&str>,
        api_key: Option<&str>,
        proj_name: Option<&str>,
        env_name: Option<&str>,
    ) -> Result<Vec<ConfigValue>> {
        let mut results: Vec<ConfigValue> = Vec::new();
        let resolve_profile: String;

        // NOTE: do profile_name first, so we can include that
        let mut value = DEFAULT_PROF_NAME.to_string();
        let mut source = SRC_DEFAULT.to_string();
        if let Some(profile_name) = profile_name {
            value = profile_name.to_string();
            source = SRC_ARG.to_string();
        } else if let Some(env_value) = ConfigEnv::get_override(CT_PROFILE) {
            value = env_value;
            source = SRC_ENV.to_string();
        }
        resolve_profile = value.clone();
        results.push(ConfigValue {
            name: PARAM_PROFILE.to_string(),
            value,
            source,
            secret: false,
            extension: false,
        });

        // get the list of profiles and values
        let profiles = Config::profile_details_list_for(&resolve_profile);

        //////////////////
        // API key
        let mut value = "".to_string();
        let mut source = "".to_string();
        if let Some(cmd_value) = api_key {
            value = cmd_value.to_string();
            source = SRC_ARG.to_string();
        } else if let Some(env_value) = ConfigEnv::get_override(CT_API_KEY) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.api_key {
                    value = prof_value.clone();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_API_KEY.to_string(),
            value,
            source,
            secret: true,
            extension: false,
        });

        //////////////////
        // Project
        let mut value = "".to_string();
        let mut source = "".to_string();
        if let Some(cmd_value) = proj_name {
            value = cmd_value.to_string();
            source = SRC_ARG.to_string();
        } else if let Some(env_value) = ConfigEnv::get_override(CT_PROJECT) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.project {
                    value = prof_value.clone();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_PROJECT.to_string(),
            value,
            source,
            secret: false,
            extension: false,
        });

        //////////////////
        // Environment
        let mut value = DEFAULT_ENV_NAME.to_string();
        let mut source = SRC_DEFAULT.to_string();
        if let Some(cmd_value) = env_name {
            value = cmd_value.to_string();
            source = SRC_ARG.to_string();
        } else if let Some(env_value) = ConfigEnv::get_override(CT_ENVIRONMENT) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.environment {
                    value = prof_value.clone();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_ENVIRONMENT.to_string(),
            value,
            source,
            secret: false,
            extension: false,
        });

        //////////////////
        // Version
        results.push(ConfigValue {
            name: PARAM_CLI_VERSION.to_string(),
            value: option_env!("CARGO_PKG_VERSION")
                .unwrap_or("Unknown")
                .to_string(),
            source: SRC_BINARY.to_string(),
            secret: false,
            extension: true,
        });

        //////////////////
        // Server URL
        let mut value = DEFAULT_SERVER_URL.to_string();
        let mut source = SRC_DEFAULT.to_string();
        if let Some(env_value) = ConfigEnv::get_override(CT_SERVER_URL) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.server_url {
                    value = prof_value.clone();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_SERVER_URL.to_string(),
            value,
            source,
            secret: false,
            extension: true,
        });

        //////////////////
        // Request timeout
        let mut value = format!("{}", DEFAULT_REQUEST_TIMEOUT);
        let mut source = SRC_DEFAULT.to_string();
        if let Some(env_value) = ConfigEnv::get_override(CT_REQ_TIMEOUT) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.request_timeout {
                    value = prof_value.clone();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_REQUEST_TIMEOUT.to_string(),
            value,
            source,
            secret: false,
            extension: true,
        });

        //////////////////
        // REST debug
        let mut value = "false".to_string();
        let mut source = SRC_DEFAULT.to_string();
        if let Some(env_value) = ConfigEnv::get_override(CT_REST_DEBUG) {
            value = env_value;
            source = SRC_ENV.to_string();
        } else {
            for profile in &profiles {
                if let Some(ref prof_value) = profile.rest_debug {
                    value = prof_value.to_string();
                    source = format!("{} ({})", SRC_PROFILE, profile.name);
                    break;
                }
            }
        }
        results.push(ConfigValue {
            name: PARAM_REST_DEBUG.to_string(),
            value,
            source,
            secret: false,
            extension: true,
        });

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    // Any tests that manipulate environment variables should be run serially as environment
    // variables are a shared global resource. Any such tests should also restore the environment
    // to its pre-test state.

    fn get_param<'a>(
        param_name: &str,
        config_values: &'a Vec<ConfigValue>,
    ) -> Option<&'a ConfigValue> {
        let mut result: Option<&ConfigValue> = None;
        for item in config_values {
            if &item.name == param_name {
                result = Some(item);
            }
        }
        result
    }

    fn sync_env(name: &str, expected: &str) {
        loop {
            if let Ok(curr) = env::var(name) {
                if expected == &curr {
                    break;
                }
            }
        }
    }

    #[allow(dead_code)]
    fn get_api_key_from_env() {
        env::set_var(CT_API_KEY, "new_key");
        sync_env(CT_API_KEY, "new_key");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.api_key, "new_key");

        let config_values = Config::get_sources(None, None, None, None).unwrap();
        let param = get_param(PARAM_API_KEY, &config_values).unwrap();
        assert_eq!(param.value, "new_key");
        assert_eq!(param.source, SRC_ENV);
        assert_eq!(param.secret, true);
        assert_eq!(param.extension, false);

        env::remove_var(CT_API_KEY);
    }

    #[test]
    fn get_version() {
        let config_values = Config::get_sources(None, None, None, None).unwrap();
        let param = get_param(PARAM_CLI_VERSION, &config_values).unwrap();
        assert_eq!(param.value, option_env!("CARGO_PKG_VERSION").unwrap());
        assert_eq!(param.source, SRC_BINARY);
        assert_eq!(param.secret, false);
        assert_eq!(param.extension, true);
    }

    #[test]
    #[serial]
    fn api_key_from_args_takes_precedent() {
        env::set_var(CT_API_KEY, "key_from_env");
        let config =
            Config::load_config(Some("key_from_args"), Some(DEFAULT_PROF_NAME), None, None)
                .unwrap();

        assert_eq!(config.api_key, "key_from_args");

        let config_values =
            Config::get_sources(Some(DEFAULT_PROF_NAME), Some("key_from_args"), None, None)
                .unwrap();
        let param = get_param(PARAM_API_KEY, &config_values).unwrap();
        assert_eq!(param.value, "key_from_args");
        assert_eq!(param.source, SRC_ARG);
        assert_eq!(param.secret, true);
        assert_eq!(param.extension, false);

        env::remove_var(CT_API_KEY)
    }

    #[test]
    #[serial]
    fn get_server_url_from_env() {
        env::set_var(CT_SERVER_URL, "http://localhost:7001/graphql");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.server_url, "http://localhost:7001/graphql");

        let config_values = Config::get_sources(Some(DEFAULT_PROF_NAME), None, None, None).unwrap();
        let param = get_param(PARAM_SERVER_URL, &config_values).unwrap();
        assert_eq!(param.value, "http://localhost:7001/graphql");
        assert_eq!(param.source, SRC_ENV);
        assert_eq!(param.secret, false);
        assert_eq!(param.extension, true);

        env::remove_var(CT_SERVER_URL);
    }

    #[test]
    #[serial]
    fn get_request_timeout_from_env() {
        env::set_var(CT_REQ_TIMEOUT, "123");
        let config = Config::load_config(None, Some(DEFAULT_PROF_NAME), None, None).unwrap();

        assert_eq!(config.request_timeout, Some(Duration::new(123, 0)));

        let config_values = Config::get_sources(Some(DEFAULT_PROF_NAME), None, None, None).unwrap();
        let param = get_param(PARAM_REQUEST_TIMEOUT, &config_values).unwrap();
        assert_eq!(param.value, "123");
        assert_eq!(param.source, SRC_ENV);
        assert_eq!(param.secret, false);
        assert_eq!(param.extension, true);

        env::remove_var(CT_REQ_TIMEOUT);
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

    #[test]
    fn cmd_args_take_precedence() {
        env::set_var(CT_PROJECT, "env_prof");
        env::set_var(CT_API_KEY, "env_key");
        env::set_var(CT_PROJECT, "env_proj");
        env::set_var(CT_ENVIRONMENT, "env_env");

        let config_values = Config::get_sources(
            Some(DEFAULT_PROF_NAME),
            Some("my_arg_key"),
            Some("proj_name"),
            Some("env_name"),
        )
        .unwrap();
        let api_key = get_param(PARAM_API_KEY, &config_values).unwrap();
        assert_eq!(api_key.value, "my_arg_key");
        assert_eq!(api_key.source, SRC_ARG);
        assert_eq!(api_key.secret, true);

        let proj = get_param(PARAM_PROJECT, &config_values).unwrap();
        assert_eq!(proj.value, "proj_name");
        assert_eq!(proj.source, SRC_ARG);
        assert_eq!(proj.secret, false);

        let prof = get_param(PARAM_PROFILE, &config_values).unwrap();
        assert_eq!(prof.value, DEFAULT_PROF_NAME);
        assert_eq!(prof.source, SRC_ARG);
        assert_eq!(prof.secret, false);

        let env = get_param(PARAM_ENVIRONMENT, &config_values).unwrap();
        assert_eq!(env.value, "env_name");
        assert_eq!(prof.source, SRC_ARG);
        assert_eq!(prof.secret, false);

        env::remove_var(CT_PROJECT);
        env::remove_var(CT_API_KEY);
        env::remove_var(CT_PROJECT);
        env::remove_var(CT_ENVIRONMENT);
    }
}
