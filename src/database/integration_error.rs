use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum IntegrationError {
    NotFound(String),
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IntegrationError::NotFound(msg) => write!(f, "{}", msg),
            IntegrationError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            IntegrationError::ResponseError(msg) => write!(f, "{}", msg),
            IntegrationError::UnhandledError(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for IntegrationError {}
