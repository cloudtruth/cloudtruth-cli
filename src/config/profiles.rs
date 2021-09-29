use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde(default)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_profile: Option<String>,
}

// TODO: Rick Porter 4/21, fix this so don't have to udpate when Profile is updated
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileDetails {
    pub api_key: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub name: String,
    pub project: Option<String>,
    pub parent: Option<String>,
    pub server_url: Option<String>,
    pub request_timeout: Option<String>,
    pub rest_debug: Option<bool>,
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
            rest_debug: None,
        }
    }
}

fn empty_to_none(value: &Option<String>) -> Option<String> {
    match value {
        Some(x) => match x.is_empty() {
            true => None,
            false => Some(x.clone()),
        },
        _ => None,
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
            rest_debug: other.rest_debug.or(self.rest_debug),
            server_url: other.server_url.clone().or_else(|| self.server_url.clone()),
            source_profile: self.source_profile.clone(),
        }
    }

    // turns any Some("") string properties into None
    pub fn remove_empty(&self) -> Profile {
        Profile {
            api_key: empty_to_none(&self.api_key),
            description: empty_to_none(&self.description),
            environment: empty_to_none(&self.environment),
            project: empty_to_none(&self.project),
            request_timeout: self.request_timeout,
            rest_debug: self.rest_debug,
            server_url: empty_to_none(&self.server_url),
            source_profile: empty_to_none(&self.source_profile),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.api_key.is_none()
            && self.description.is_none()
            && self.environment.is_none()
            && self.project.is_none()
            && self.request_timeout.is_none()
            && self.rest_debug.is_none()
            && self.server_url.is_none()
            && self.source_profile.is_none()
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
            rest_debug: Some(true),
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
            rest_debug: Some(false),
            server_url: Some("http://localhost:7001/graphql".to_string()),
            ..Profile::default()
        };

        let second = Profile {
            ..Profile::default()
        };

        assert_eq!(first, first.merge(&second));
    }

    #[test]
    fn empty_test() {
        let prof: Profile = Profile::default();
        assert!(prof.is_empty());

        let prof = Profile {
            rest_debug: Some(false),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            request_timeout: Some(1024),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            api_key: Some("abc123".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            description: Some("my description".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            environment: Some("ename".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            project: Some("proj".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            server_url: Some("url".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);

        let prof = Profile {
            source_profile: Some("source-profile".to_string()),
            ..Profile::default()
        };
        assert_eq!(prof.is_empty(), false);
    }

    #[test]
    fn remove_empty() {
        let prof = Profile {
            api_key: Some("".to_string()),
            description: Some("".to_string()),
            environment: Some("".to_string()),
            project: Some("".to_string()),
            request_timeout: None,
            rest_debug: None,
            server_url: Some("".to_string()),
            source_profile: Some("".to_string()),
        };

        let prof2 = prof.remove_empty();
        assert_eq!(prof.is_empty(), false);
        assert_eq!(prof2.is_empty(), true);

        let prof = Profile {
            api_key: Some("api-key".to_string()),
            description: Some("desc".to_string()),
            environment: Some("env".to_string()),
            project: Some("proj".to_string()),
            request_timeout: None,
            rest_debug: None,
            server_url: Some("url".to_string()),
            source_profile: Some("src-prof".to_string()),
        };
        let prof2 = prof.remove_empty();
        assert_eq!(prof, prof2);
    }
}
