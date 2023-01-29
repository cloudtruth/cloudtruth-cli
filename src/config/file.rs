use crate::config::file_errors::ConfigFileError;
use crate::config::profiles::{Profile, ProfileDetails};
use crate::config::update::Updates;
use color_eyre::eyre::Result;
use indoc::indoc;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct ConfigFile {
    updates: Option<Updates>,
    profiles: HashMap<String, Profile>,
}

const HDR_UPDATES: &str = "updates";
const HDR_PROFILES: &str = "profiles";

type ConfigFileResult<T> = std::result::Result<T, ConfigFileError>;

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
            updates:
              check: true

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
        let config_file: ConfigFile = serde_yaml::from_str(config)?;

        let profile = config_file.profiles.get(profile_name);
        if let Some(profile) = profile {
            Self::resolve_source_profile_chain(
                &config_file,
                profile,
                profile_name,
                &mut vec![profile_name.to_string()],
            )
        } else {
            Err(ConfigFileError::ProfileNameNotFound(
                profile_name.to_string(),
            ))
        }
    }

    /// Checks that the provided content is valid YAML
    pub(crate) fn validate_content(content: &str) -> ConfigFileResult<()> {
        let config_file: ConfigFile = serde_yaml::from_str(content)?;

        // attempt to load all the profiles, since that's when we detect circular dependencies
        for profile in config_file.profiles.keys() {
            let _ = Self::load_profile(content, profile)?;
        }
        Ok(())
    }

    /// Gets the text associated with the specified profile (including comments)
    fn get_profile_text(content: &str, profile_name: &str) -> String {
        // TODO: solve this with a regex
        let config_lines: Vec<&str> = content.split('\n').collect();
        let mut prof_lines: Vec<&str> = vec![];
        let indent = " ".repeat(2);
        let indent_plus = format!("{indent} ");
        let needle = format!("{indent}{profile_name}:");
        let mut start = false;
        let mut prof_start = false;
        for line in config_lines {
            if !prof_start {
                prof_start = line.starts_with(HDR_PROFILES);
                continue;
            }
            if line.starts_with(&needle) {
                prof_lines.push(line);
                start = true;
                continue;
            }
            if !start {
                continue;
            }
            // if we hit the next key (or comment), we're done
            if line.starts_with(&indent) && !line.starts_with(&indent_plus) {
                break;
            }
            // if we're outside the current profile, we're done
            if !line.is_empty() && !line.starts_with(&indent) {
                break;
            }
            prof_lines.push(line);
        }
        prof_lines.join("\n")
    }

    pub fn set_profile(
        config: &str,
        profile_name: &str,
        api_key: Option<&str>,
        description: Option<&str>,
        environment: Option<&str>,
        project: Option<&str>,
        source: Option<&str>,
    ) -> ConfigFileResult<String> {
        let result: String;
        let mut config_file: ConfigFile = serde_yaml::from_str(config)?;
        let new_prof = Profile {
            api_key: api_key.map(String::from),
            description: description.map(String::from),
            environment: environment.map(String::from),
            project: project.map(String::from),
            request_timeout: None,
            rest_debug: None,
            rest_success: vec![],
            rest_page_size: None,
            server_url: None,
            source_profile: source.map(String::from),
            accept_invalid_certs: None,
        };

        let profiles = config_file.profiles.borrow_mut();
        if let Some(profile) = profiles.get_mut(profile_name) {
            let orig_text = ConfigFile::get_profile_text(config, profile_name);
            let merged = profile.merge(&new_prof).remove_empty();
            let mut new_text: String;
            if merged == *profile {
                // do nothing here, so we don't lose comments, and reorder keys
                new_text = orig_text.clone();
            } else {
                profiles.insert(profile_name.to_string(), merged);
                let new_file = serde_yaml::to_string(&config_file)?;
                new_text = ConfigFile::get_profile_text(&new_file, profile_name);
                if !new_text.ends_with('\n') {
                    new_text.push('\n');
                }
            }
            result = config.replace(&orig_text, &new_text);
        } else if !new_prof.is_empty() {
            profiles.insert(profile_name.to_string(), new_prof);
            let new_file = serde_yaml::to_string(&config_file)?;
            let mut new_text = ConfigFile::get_profile_text(&new_file, profile_name);
            if !new_text.ends_with('\n') {
                new_text.push('\n')
            }
            result = format!("{config}\n{new_text}");
        } else {
            // no changes
            result = config.to_string();
        }
        Ok(result)
    }

    pub fn remove_profile(config: &str, profile_name: &str) -> Result<String> {
        let profile_text = ConfigFile::get_profile_text(config, profile_name);
        let new_file = config.replace(&profile_text, "");
        Ok(new_file)
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
            request_timeout: profile.request_timeout.map(|t| format!("{t}")),
            rest_debug: profile.rest_debug,
            rest_success: profile.rest_success.clone(),
            rest_page_size: profile.rest_page_size,
            accept_invalid_certs: profile.accept_invalid_certs,
        }
    }

    pub(crate) fn get_profile_details(config: &str) -> ConfigFileResult<Vec<ProfileDetails>> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if !config.is_empty() {
            let config_file: ConfigFile = serde_yaml::from_str(config)?;
            profiles = config_file
                .profiles
                .iter()
                .map(|(k, v)| ConfigFile::create_project_details(k, v))
                .collect();
        }

        Ok(profiles)
    }

    pub(crate) fn get_details_by_name(
        config: &str,
        profile_name: &str,
    ) -> ConfigFileResult<Option<ProfileDetails>> {
        let mut details: Option<ProfileDetails> = None;
        if !config.is_empty() {
            let config_file: ConfigFile = serde_yaml::from_str(config)?;
            if let Some(cfg_prof) = config_file.profiles.get(profile_name) {
                details = Some(ConfigFile::create_project_details(profile_name, cfg_prof));
            }
        }
        Ok(details)
    }

    /// Gets an ordered list of `ProfileDetails` for the specified `profile_name` and it's parents.
    pub(crate) fn get_details_for(
        config: &str,
        profile_name: &str,
    ) -> ConfigFileResult<Vec<ProfileDetails>> {
        let mut profiles: Vec<ProfileDetails> = Vec::new();
        if !config.is_empty() {
            let config_file: ConfigFile = serde_yaml::from_str(config)?;
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

    fn get_update_text(content: &str) -> String {
        // TODO: solve this with a regex
        let config_lines: Vec<&str> = content.split('\n').collect();
        let mut update_lines: Vec<&str> = vec![];
        let indent = " ".repeat(2);

        let mut start = false;
        for line in config_lines {
            if !start {
                start = line.starts_with(HDR_UPDATES);
                if start {
                    update_lines.push(line);
                }
                continue;
            }

            // if we hit the next key (or comment), we're done
            if !line.is_empty() && !line.starts_with(&indent) {
                break;
            }

            update_lines.push(line);
        }
        update_lines.join("\n")
    }

    pub fn load_updates(config: &str) -> ConfigFileResult<Updates> {
        let config_file: ConfigFile = serde_yaml::from_str(config)?;
        if let Some(updates) = config_file.updates {
            Ok(updates)
        } else {
            Ok(Updates::default())
        }
    }

    pub fn set_updates(config: &str, updates: &Updates) -> ConfigFileResult<String> {
        let current = ConfigFile::get_update_text(config);
        let mut next = serde_yaml::to_string(&updates)?;
        let replacement = format!("{}{}", "\n", " ".repeat(2));
        let header = format!("{HDR_UPDATES}:");
        next = next.replace('\n', &replacement).replace("---", &header);
        let result = if current.is_empty() {
            format!("{config}\n{next}")
        } else {
            config.replace(&current, &next)
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::file::{ConfigFile, ConfigFileError};
    use crate::config::profiles::ProfileDetails;
    use crate::config::{Action, Frequency, Updates};
    use assert_matches::assert_matches;
    use chrono::NaiveDate;
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
        assert_eq!(Some("read_only_key".to_string()), profile.api_key);
        let result = ConfigFile::validate_content(config);
        assert!(result.is_ok());
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
        );
        let result = ConfigFile::validate_content(config);
        assert!(result.is_ok());
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
        assert_eq!(Some(50), profile.request_timeout);
        let result = ConfigFile::validate_content(config);
        assert!(result.is_ok());
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
        let err_msg = format!("{error}");
        assert!(err_msg.contains("profiles.default.request_timeout: invalid type"));

        let error = ConfigFile::validate_content(config).unwrap_err();
        let err_msg = format!("{error}");
        assert!(err_msg.contains("profiles.default.request_timeout: invalid type"));
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

        let error = ConfigFile::validate_content(config).unwrap_err();
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

        let result = ConfigFile::validate_content(config);
        assert!(result.is_ok());
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

        // NOTE: due to order loading, we cannot guarantee vector order or profile name
        let error = ConfigFile::validate_content(config).unwrap_err();
        let err_msg = format!("{error}");
        assert!(err_msg
            .contains("Your configuration file has a cycle source_profile cycle for profile"));
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
        for value in ["default", "read-only", "invalid_url"] {
            assert!(profile_names.iter().any(|s| s == value))
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
        for value in [
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

    #[test]
    fn profile_details_by_name_found() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            grandparent-profile:
                api_key: grandparent_key
        "#
        );

        let result = ConfigFile::get_details_by_name(config, "grandparent-profile");
        assert!(result.is_ok());
        let maybe_profile: Option<ProfileDetails> = result.unwrap();
        assert!(maybe_profile.is_some());
        let profile = maybe_profile.unwrap();
        assert_eq!(
            profile,
            ProfileDetails {
                name: "grandparent-profile".to_string(),
                api_key: Some("grandparent_key".to_string()),
                description: None,
                environment: None,
                parent: None,
                project: None,
                server_url: None,
                rest_debug: None,
                rest_success: vec![],
                rest_page_size: None,
                request_timeout: None,
                accept_invalid_certs: None
            },
        );
    }

    #[test]
    fn profile_details_by_name_not_found() {
        let config = indoc!(
            r#"
        profiles:
            default:
                api_key: default_key
                server_url: http://localhost:7001/graphql

            grandparent-profile:
                api_key: grandparent_key
        "#
        );

        let result = ConfigFile::get_details_by_name(config, "missing-profile");
        assert!(result.is_ok());
        let maybe_profile: Option<ProfileDetails> = result.unwrap();
        assert!(maybe_profile.is_none());
    }

    #[test]
    fn profile_set_add() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # This includes a comment
            api_key: default_key
            server_url: http://localhost:7001/graphql
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            # This includes a comment
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            api_key: grandparent_key
        "#
        );

        let result = ConfigFile::set_profile(
            config,
            "grandparent-profile",
            Some("grandparent_key"),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn profile_set_update_replace_key() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # This includes a comment
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            # This includes a comment
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            api_key: my_new_key
            server_url: "https://localhost:8000"
        "#
        );

        let result = ConfigFile::set_profile(
            config,
            "grandparent-profile",
            Some("my_new_key"),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn profile_set_update_add_project() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            api_key: default_key
            project: YourFirstProject
            server_url: "http://localhost:7001/graphql"

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );

        let result = ConfigFile::set_profile(
            config,
            "default",
            None,
            None,
            None,
            Some("YourFirstProject"),
            None,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn profile_set_missing_source_added() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key

          new-profile-name:
            source_profile: are-you-my-mother
        "#
        );
        let result = ConfigFile::set_profile(
            config,
            "new-profile-name",
            None,
            None,
            None,
            None,
            Some("are-you-my-mother"),
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn profile_set_empty_not_added() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let result =
            ConfigFile::set_profile(config, "new-profile-name", None, None, None, None, None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, config);
    }

    #[test]
    fn profile_set_unchanged() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            server_url: https://localhost:8000
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let result = ConfigFile::set_profile(
            config,
            "default",
            Some("default_key"),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, config);
    }

    #[test]
    fn profile_set_remains() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile: {}
        "#
        );
        let result = ConfigFile::set_profile(
            config,
            "grandparent-profile",
            Some(""),
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn profile_delete() {
        let config = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

          grandparent-profile:
            # Comments get lost, keys reordered, and quotes maybe added -- reasonable first pass
            api_key: grandparent_key
        "#
        );
        let expected = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

        "#
        );
        let result = ConfigFile::remove_profile(config, "grandparent-profile");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn add_updates() {
        let original = indoc!(
            r#"
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

        "#
        );

        let updates = Updates {
            check: true,
            action: None,
            last_checked: None,
            frequency: None,
        };
        let expected = format!("{original}\nupdates:\n  check: true\n  ");
        let result = ConfigFile::set_updates(original, &updates);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn update_updates() {
        let original = indoc!(
            r#"
        updates:
          check: false

        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

        "#
        );
        let expected = indoc!(
            r#"
        updates:
          check: true
          action: Error
          last_checked: 2021-01-20
          frequency: Monthly
          
        profiles:
          default:
            # comment lost -- not ideal
            api_key: default_key
            server_url: http://localhost:7001/graphql

        "#
        );

        let updates = Updates {
            check: true,
            action: Some(Action::Error),
            last_checked: Some(NaiveDate::from_ymd(2021, 1, 20)),
            frequency: Some(Frequency::Monthly),
        };

        let result = ConfigFile::set_updates(original, &updates);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
