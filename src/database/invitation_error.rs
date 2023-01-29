use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum InvitationError {
    InvalidRole(String),
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for InvitationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvitationError::InvalidRole(role) => write!(f, "Invalid role value: '{role}'"),
            InvitationError::Authentication(msg) => write!(f, "Not Authenticated: {msg}"),
            InvitationError::ResponseError(msg) => write!(f, "{msg}"),
            InvitationError::UnhandledError(msg) => write!(f, "Unhandled error: {msg}"),
        }
    }
}

impl error::Error for InvitationError {}
