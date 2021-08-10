use crate::cli;
use crate::config::profiles::{Profile, ProfileDetails};
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
            # You can have multiple profiles to group your configuration. E.g., if you use multiple
            # projects, you can create two separate profiles each with its own role appropriate API
            # key. When you invoke the CloudTruth CLI tool, you can pass an argument to choose
            # which profile to load. Profiles can inherit values from other profiles by using the
            # `source_profile` setting, providing it with the name of another profile. Profile
            # chains can be arbitrarily deep, but may not contain cycles.

            profiles:
              default:
                api_key: ""
                description: Default environment/project

              # another-profile:
              #   source_profile: default
              #   api_key: "my-read-only-api-key"
              #   description: Read-only user on a different project
              #   project: other-project-name
              #   environment: pre-production
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

    fn create_project_details(name: &str, profile: &Profile) -> ProfileDetails {
        ProfileDetails {
            api_key: profile.api_key.clone(),
            description: profile.description.clone(),
            environment: profile.environment.clone(),
            name: name.to_string(),
            project: profile.project.clone(),
            parent: profile.source_profile.clone(),
            server_url: profile.server_url.clone(),
            request_timeout: profile.request_timeout.map(|t| format!("{}", t)),
            rest_debug: profile.rest_debug,
        }
    }

    pub(crate) fn get_profile_details(config: &str) -> ConfigFileResult<Vec<ProfileDetails>> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if !config.is_empty() {
            let config_file: ConfigFile = serde_yaml::from_str(&config)?;
            profiles = config_file
                .profiles
                .iter()
                .map(|(k, v)| ConfigFile::create_project_details(k, v))
                .collect();
        }

        Ok(profiles)
    }

    /// Gets an ordered list of `ProfileDetails` for the specified `profile_name` and it's parents.
    pub(crate) fn get_details_for(
        config: &str,
        profile_name: &str,
    ) -> ConfigFileResult<Vec<ProfileDetails>> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if !config.is_empty() {
            let config_file: ConfigFile = serde_yaml::from_str(&config)?;
            let mut prof_name = profile_name.to_string();
            while let Some(profile) = config_file.profiles.get(&prof_name) {
                profiles.push(ConfigFile::create_project_details(&prof_name, profile));
                if let Some(ref parent) = profile.source_profile {
                    prof_name = parent.clone();
                } else {
                    break;
                }
            }
        }

        Ok(profiles)
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
                let pre_exists = cycle.contains(source_profile_name);

                // Always add the value -- even for the error case, so we can show the complete
                // cycle in the error object
                cycle.push(source_profile_name.clone());

                if pre_exists {
                    Err(ConfigFileError::SourceProfileCyclicError(
                        original_profile_name.to_string(),
                        cycle.clone(),
                    ))
                } else {
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
    fn get_request_timeout_from_profile() {
        let config = indoc!(
            r#"
        profiles:
            default:
                request_timeout: 50
        "#
        );

        let profile = ConfigFile::load_profile(config, "default").unwrap();
        assert_eq!(Some(50), profile.request_timeout)
    }

    #[test]
    fn invalid_request_timeout_from_profile() {
        let config = indoc!(
            r#"
        profiles:
            default:
                request_timeout: not an integer
        "#
        );

        let error = ConfigFile::load_profile(config, "default").unwrap_err();
        let err_msg = format!("{}", error);
        assert!(err_msg.contains("Your configuration file is not syntactically valid YAML."));
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

    #[test]
    fn profile_names_basic() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            read-only:
                api_key: read_only_key

            invalid_url:
                server_url: foobar.com
        "#
        );

        let result = ConfigFile::get_profile_details(config);
        assert!(result.is_ok());
        let profile_names: Vec<String> = result.unwrap().iter().map(|v| v.name.clone()).collect();
        for value in vec!["default", "read-only", "invalid_url"] {
            assert!(profile_names
                .iter()
                .find(|&s| *s == value.to_string())
                .is_some())
        }
    }

    #[test]
    fn profile_names_empty() {
        let config = indoc!("");
        let result = ConfigFile::get_profile_details(config);
        assert!(&result.is_ok());
        let profile_names = result.unwrap();
        assert!(&profile_names.is_empty());
    }

    #[test]
    fn profile_details_list() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            grandparent-profile:
                api_key: grandparent_key
                server_url: http://localhost/api

            my-profile:
                api_key: my_key
                source_profile: parent-profile

            parent-profile:
                source_profile: grandparent-profile
                request_timeout: 300
        "#
        );

        let result = ConfigFile::get_profile_details(config);
        assert!(result.is_ok());
        let list = result.unwrap();
        for value in vec![
            "my-profile",
            "parent-profile",
            "grandparent-profile",
            "default",
        ] {
            let search = list.iter().find(|&d| d.name.as_str() == value);
            assert!(search.is_some());
            let item = search.unwrap();
            match item.name.as_str() {
                "my-profile" => {
                    assert_eq!(item.parent.clone().unwrap().as_str(), "parent-profile");
                    assert_eq!(item.api_key.clone().unwrap().as_str(), "my_key");
                    assert_eq!(item.server_url, None);
                }
                "parent-profile" => {
                    assert_eq!(item.parent.clone().unwrap().as_str(), "grandparent-profile");
                    assert_eq!(item.server_url, None);
                    assert_eq!(item.api_key, None);
                    assert_eq!(item.request_timeout.clone().unwrap().as_str(), "300");
                }
                "grandparent-profile" => {
                    assert_eq!(item.parent, None);
                    assert_eq!(
                        item.server_url.clone().unwrap().as_str(),
                        "http://localhost/api"
                    );
                    assert_eq!(item.api_key.clone().unwrap().as_str(), "grandparent_key");
                    assert_eq!(item.request_timeout, None);
                }
                // no other checks
                _ => {}
            }
        }
    }

    #[test]
    fn profile_details_for() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            grandparent-profile:
                api_key: grandparent_key

            my-profile:
                api_key: my_key
                source_profile: parent-profile

            parent-profile:
                source_profile: grandparent-profile
        "#
        );

        let result = ConfigFile::get_details_for(config, "my-profile");
        assert!(result.is_ok());
        let profile_names: Vec<String> = result.unwrap().iter().map(|v| v.name.clone()).collect();
        assert_eq!(
            profile_names,
            vec![
                "my-profile".to_string(),
                "parent-profile".to_string(),
                "grandparent-profile".to_string()
            ]
        );
    }
}
