use serde_yaml::Error;
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

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
            ConfigFileError::MalformedConfigFile(source_error) => {
                write!(f, "YAML error: {}", source_error)
            }
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
