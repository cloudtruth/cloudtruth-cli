use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ApiError {
    Authentication(String),
    ResponseError(String),
    UnsupportedFormat(String),
    UnhandledError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            ApiError::ResponseError(msg) => write!(f, "{}", msg),
            ApiError::UnsupportedFormat(format) => write!(f, "Invalid format: {}", format),
            ApiError::UnhandledError(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
}

impl error::Error for ApiError {}
