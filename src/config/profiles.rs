use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Profile {
    pub api_key: Option<String>,
    pub server_url: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_key: None,
            server_url: None,
        }
    }
}

impl Profile {
    #[must_use]
    pub(crate) fn merge(&self, other: &Self) -> Profile {
        Profile {
            api_key: other.api_key.clone().or_else(|| self.api_key.clone()),
            server_url: other.server_url.clone().or_else(|| self.server_url.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::profiles::Profile;

    #[test]
    fn merged_values_take_priority() {
        let first = Profile {
            api_key: None,
            server_url: None,
        };

        let second = Profile {
            api_key: Some("new_key".to_string()),
            server_url: Some("http://localhost:7001/graphql".to_string()),
        };

        assert_eq!(second, first.merge(&second));
    }

    #[test]
    fn merged_empty_values_are_ignored() {
        let first = Profile {
            api_key: Some("new_key".to_string()),
            server_url: Some("http://localhost:7001/graphql".to_string()),
        };

        let second = Profile {
            api_key: None,
            server_url: None,
        };

        assert_eq!(first, first.merge(&second));
    }
}
