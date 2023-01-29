use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ProjectError {
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::Authentication(msg) => write!(f, "Not Authenticated: {msg}"),
            ProjectError::ResponseError(msg) => write!(f, "{msg}"),
            ProjectError::UnhandledError(msg) => write!(f, "Unhandled error: {msg}"),
        }
    }
}

impl error::Error for ProjectError {}
