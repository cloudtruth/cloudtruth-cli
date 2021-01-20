use crate::cli;
use crate::config::profiles::Profile;
use color_eyre::eyre::Result;
use core::fmt;
use indoc::indoc;
use serde::Deserialize;
use serde_yaml::Error;
use std::collections::HashMap;
use std::error;
use std::fmt::Formatter;
use std::sync::Arc;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigFile {
    profiles: HashMap<String, Profile>,
}

type ConfigFileResult<T> = std::result::Result<T, ConfigFileError>;

#[derive(Clone, Debug)]
pub enum ConfigFileError {
    MalformedConfigFile(Arc<serde_yaml::Error>),
    ProfileNameNotFound(String),
    SourceProfileCyclicError(String, Vec<String>),
    SourceProfileNameNotFound(String, String),
}

impl error::Error for ConfigFileError {}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ConfigFileError::MalformedConfigFile(_source_error) => write!(f, "Your configuration file is not syntactically valid YAML. Please run '{} config edit' to fix.", cli::binary_name()),
            ConfigFileError::ProfileNameNotFound(profile_name) => write!(
                f,
                "Profile '{}' does not exist in your configuration file",
                profile_name
            ),
            ConfigFileError::SourceProfileCyclicError(profile_name, cycle) => write!(
                f,
                "Your configuration file has a cycle source_profile cycle for profile '{}': {}",
                profile_name,
                cycle.join(" -> ")
            ),
            ConfigFileError::SourceProfileNameNotFound(profile_name, source_profile_name) => {
                write!(
                    f,
                    "Profile '{}' references non-existant source profile '{}'",
                    profile_name, source_profile_name
                )
            }
        }
    }
}

impl From<serde_yaml::Error> for ConfigFileError {
    fn from(error: Error) -> Self {
        ConfigFileError::MalformedConfigFile(Arc::new(error))
    }
}

impl ConfigFile {
    pub(crate) fn config_file_template() -> &'static str {
        indoc!(
            r#"
            # You can have multiple profiles to group your configuration. E.g., if you belong to
            # multiple organizations, you can create two separate profiles each with its own API
            # key. When you invoke the CloudTruth CLI tool, you can pass an argument to choose
            # which profile to load. Profiles can inherit values from other profiles by using the
            # `source_profile` setting, providing it with the name of another profile. Profile
            # chains can be arbitrarily deep, but may not contain cycles.

            profiles:
              default:
                api_key: ""

              # another-profile:
              #   source_profile: default
              #   api_key: "my-read-only-api-key"
        "#
        )
    }

    pub(crate) fn load_profile(config: &str, profile_name: &str) -> ConfigFileResult<Profile> {
        let config_file: ConfigFile = serde_yaml::from_str(&config)?;

        let profile = config_file.profiles.get(profile_name);
        if let Some(profile) = profile {
            Self::resolve_source_profile_chain(
                &config_file,
                &profile,
                profile_name,
                &mut vec![profile_name.to_string()],
            )
        } else {
            Err(ConfigFileError::ProfileNameNotFound(
                profile_name.to_string(),
            ))
        }
    }

    fn resolve_source_profile_chain(
        config_file: &ConfigFile,
        profile: &Profile,
        original_profile_name: &str,
        cycle: &mut Vec<String>,
    ) -> ConfigFileResult<Profile> {
        let source_profile_name = &profile.source_profile;

        if let Some(source_profile_name) = source_profile_name {
            let source_profile = config_file.profiles.get(source_profile_name);

            if let Some(source_profile) = source_profile {
                if cycle.contains(source_profile_name) {
                    // Although the cycle is already detected, add the value so we can show the
                    // complete cycle in the error object.
                    cycle.push(source_profile_name.clone());

                    Err(ConfigFileError::SourceProfileCyclicError(
                        original_profile_name.to_string(),
                        cycle.clone(),
                    ))
                } else {
                    cycle.push(source_profile_name.clone());

                    Self::resolve_source_profile_chain(
                        config_file,
                        &source_profile.merge(profile),
                        original_profile_name,
                        cycle,
                    )
                }
            } else {
                Err(ConfigFileError::SourceProfileNameNotFound(
                    original_profile_name.to_string(),
                    source_profile_name.clone(),
                ))
            }
        } else {
            let default_profile = config_file.profiles.get("default");
            if let Some(default_profile) = default_profile {
                Ok(default_profile.merge(profile))
            } else {
                Ok(profile.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::file::{ConfigFile, ConfigFileError};
    use assert_matches::assert_matches;
    use indoc::indoc;

    #[test]
    fn default_config_file() {
        let profile =
            ConfigFile::load_profile(ConfigFile::config_file_template(), "default").unwrap();
        assert_eq!(Some("".to_string()), profile.api_key);
    }

    #[test]
    fn get_api_key_from_profile() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key

            read-only:
                api_key: read_only_key
        "#
        );

        let profile = ConfigFile::load_profile(config, "read-only").unwrap();
        assert_eq!(Some("read_only_key".to_string()), profile.api_key)
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

        let profile = ConfigFile::load_profile(config, "default").unwrap();
        assert_eq!(
            Some("http://localhost:7001/graphql".to_string()),
            profile.server_url
        )
    }

    #[test]
    fn load_profile_with_bad_name() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: my_key
            "#
        );

        let error = ConfigFile::load_profile(config, "non-match").unwrap_err();

        assert_matches!(error, ConfigFileError::ProfileNameNotFound(profile_name) => {
            assert_eq!("non-match", profile_name)
        });
    }

    #[test]
    fn profiles_implicitly_inherit_from_default_profile() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            read-only:
                api_key: read_only_key
        "#
        );

        let profile = ConfigFile::load_profile(config, "read-only").unwrap();
        assert_eq!(Some("read_only_key".to_string()), profile.api_key);
        assert_eq!(
            Some("http://localhost:7001/graphql".to_string()),
            profile.server_url
        );
    }

    #[test]
    fn profile_without_source_profile_and_without_default_profile() {
        let config = indoc!(
            r#"
        profiles:
            read-only:
                api_key: read_only_key
        "#
        );

        let profile = ConfigFile::load_profile(config, "read-only").unwrap();
        assert_eq!(Some("read_only_key".to_string()), profile.api_key);
        assert_eq!(None, profile.server_url);
    }

    #[test]
    fn profile_referencing_invalid_source_profile() {
        let config = indoc!(
            r#"
        profiles:
            read-only:
                api_key: read_only_key
                source_profile: non-existent
        "#
        );

        let error = ConfigFile::load_profile(config, "read-only").unwrap_err();

        assert_matches!(error, ConfigFileError::SourceProfileNameNotFound(profile_name, source_profile_name) => {
            assert_eq!("read-only", profile_name);
            assert_eq!("non-existent", source_profile_name);
        });
    }

    #[test]
    fn profile_source_chain_followed() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            parent-profile:
                api_key: parent_key

            my-profile:
                source_profile: parent-profile
        "#
        );

        let profile = ConfigFile::load_profile(config, "my-profile").unwrap();
        assert_eq!(Some("parent_key".to_string()), profile.api_key);
        assert_eq!(
            Some("http://localhost:7001/graphql".to_string()),
            profile.server_url
        );
    }

    #[test]
    fn source_profile_cycle() {
        let config = indoc!(
            r#"
        profiles:
            a:
                source_profile: b

            b:
                source_profile: c

            c:
                source_profile: a
        "#
        );

        let error = ConfigFile::load_profile(config, "c").unwrap_err();

        assert_matches!(error, ConfigFileError::SourceProfileCyclicError(profile_name, cycle) => {
            assert_eq!("c", profile_name);
            assert_eq!(vec!["c", "a", "b", "c"], cycle);
        });
    }
}
