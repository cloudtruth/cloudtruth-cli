use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum GroupError {
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GroupError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            GroupError::ResponseError(msg) => write!(f, "{}", msg),
            GroupError::UnhandledError(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
}

impl error::Error for GroupError {}
