use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum UserError {
    InvalidRole(String),
    MembershipNotFound(),
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserError::InvalidRole(role) => write!(f, "Invalid role value: '{}'", role),
            UserError::MembershipNotFound() => write!(f, "Membership not found"),
            UserError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            UserError::ResponseError(msg) => write!(f, "{}", msg),
            UserError::UnhandledError(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
}

impl error::Error for UserError {}
