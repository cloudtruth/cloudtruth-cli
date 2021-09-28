use cloudtruth_restapi::apis::projects_api::{
    ProjectsParametersValuesCreateError, ProjectsParametersValuesPartialUpdateError,
};
use cloudtruth_restapi::apis::Error;
use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ParameterError {
    CreateValueError(Error<ProjectsParametersValuesCreateError>),
    UpdateValueError(Error<ProjectsParametersValuesPartialUpdateError>),
    InvalidFqnOrJmesPath(String),
    RuleViolation(String),
    RuleError(String, String),
    UnhandledError(String),
    ResponseError(String),
}

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParameterError::InvalidFqnOrJmesPath(msg) => {
                write!(f, "Invalid FQN or JMES path expression: {}", msg)
            }
            ParameterError::RuleViolation(msg) => {
                write!(f, "Rule violation: {}", msg)
            }
            ParameterError::RuleError(action, msg) => {
                write!(f, "Rule {} error: {}", action, msg.replace("_len", "-len"))
            }
            ParameterError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
            ParameterError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for ParameterError {}
