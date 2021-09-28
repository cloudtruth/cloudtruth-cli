use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum EnvironmentError {
    Authentication(String),
    DeleteNotAllowed(String),
    NotFound(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentError::Authentication(msg) => {
                write!(f, "Not Authenticated: {}", msg)
            }
            EnvironmentError::DeleteNotAllowed(msg) => {
                write!(f, "Delete not allowed: {}", msg)
            }
            EnvironmentError::NotFound(name) => {
                write!(f, "Did not find environment '{}'", name)
            }
            EnvironmentError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
            EnvironmentError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
        }
    }
}

impl error::Error for EnvironmentError {}
