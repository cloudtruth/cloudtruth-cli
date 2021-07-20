use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Profile {
    pub api_key: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub project: Option<String>,
    pub request_timeout: Option<u64>,
    pub server_url: Option<String>,
    pub(crate) source_profile: Option<String>,
}

// TODO: Rick Porter 4/21, fix this so don't have to udpate when Profile is updated
#[derive(Clone, Debug)]
pub struct ProfileDetails {
    pub api_key: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub name: String,
    pub project: Option<String>,
    pub parent: Option<String>,
    pub server_url: Option<String>,
    pub request_timeout: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_key: None,
            description: None,
            environment: None,
            project: None,
            request_timeout: None,
            server_url: None,
            source_profile: None,
        }
    }
}

impl Profile {
    #[must_use]
    pub(crate) fn merge(&self, other: &Self) -> Profile {
        Profile {
            api_key: other.api_key.clone().or_else(|| self.api_key.clone()),
            description: other
                .description
                .clone()
                .or_else(|| self.description.clone()),
            environment: other
                .environment
                .clone()
                .or_else(|| self.environment.clone()),
            project: other.project.clone().or_else(|| self.project.clone()),
            request_timeout: other.request_timeout.or(self.request_timeout),
            server_url: other.server_url.clone().or_else(|| self.server_url.clone()),
            source_profile: self.source_profile.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::profiles::Profile;

    #[test]
    fn merged_values_take_priority() {
        let first = Profile {
            ..Profile::default()
        };

        let second = Profile {
            api_key: Some("new_key".to_string()),
            description: Some("describe your param in 25 words or less".to_string()),
            environment: Some("my_environment".to_string()),
            project: Some("skunkworks".to_string()),
            request_timeout: Some(100),
            server_url: Some("http://localhost:7001/graphql".to_string()),
            ..Profile::default()
        };

        assert_eq!(second, first.merge(&second));
    }

    #[test]
    fn merged_empty_values_are_ignored() {
        let first = Profile {
            api_key: Some("new_key".to_string()),
            description: Some("describe your param in 25 words or less".to_string()),
            environment: Some("my_environment".to_string()),
            project: Some("skunkworks".to_string()),
            request_timeout: Some(23),
            server_url: Some("http://localhost:7001/graphql".to_string()),
            ..Profile::default()
        };

        let second = Profile {
            ..Profile::default()
        };

        assert_eq!(first, first.merge(&second));
    }
}
