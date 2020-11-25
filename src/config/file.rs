use crate::config::profiles::Profile;
use color_eyre::eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ConfigFile {
    profiles: HashMap<String, Profile>,
}

impl ConfigFile {
    pub(crate) fn load_profile(config: &str, profile_name: &str) -> Result<Option<Profile>> {
        let config_map: ConfigFile = serde_yaml::from_str(&config)?;
        let profile = config_map.profiles.get(profile_name);

        Ok(profile.cloned())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::file::ConfigFile;
    use indoc::indoc;

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
        assert_eq!(Some("read_only_key".to_string()), profile.unwrap().api_key)
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
            profile.unwrap().server_url
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

        let profile = ConfigFile::load_profile(config, "non-matcch").unwrap();
        assert!(profile.is_none())
    }
}
