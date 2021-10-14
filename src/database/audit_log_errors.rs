use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum AuditLogError {
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for AuditLogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuditLogError::Authentication(msg) => {
                write!(f, "Not Authenticated: {}", msg)
            }
            AuditLogError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
            AuditLogError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
        }
    }
}

impl error::Error for AuditLogError {}
