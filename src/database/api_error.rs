use serde_json::Error;
use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ApiError {
    Authentication(String),
    MalformedApiFile(serde_json::Error),
    ResponseError(String),
    UnsupportedFormat(String),
    UnhandledError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            ApiError::MalformedApiFile(e) => {
                write!(f, "Invalid API specification: {}", e.to_string())
            }
            ApiError::ResponseError(msg) => write!(f, "{}", msg),
            ApiError::UnsupportedFormat(format) => write!(f, "Invalid format: {}", format),
            ApiError::UnhandledError(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: Error) -> Self {
        Self::MalformedApiFile(e)
    }
}

impl error::Error for ApiError {}
