use std::error;
use std::fmt;
use std::fmt::Formatter;

use super::UserError;

#[derive(Debug)]
pub enum GroupError {
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
    UserError(UserError),
}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GroupError::Authentication(msg) => write!(f, "Not Authenticated: {msg}"),
            GroupError::ResponseError(msg) => write!(f, "{msg}"),
            GroupError::UnhandledError(msg) => write!(f, "Unhandled error: {msg}"),
            GroupError::UserError(user_error) => user_error.fmt(f),
        }
    }
}

impl From<UserError> for GroupError {
    fn from(user_error: UserError) -> Self {
        match user_error {
            UserError::Authentication(msg) => GroupError::Authentication(msg),
            UserError::ResponseError(msg) => GroupError::ResponseError(msg),
            UserError::UnhandledError(msg) => GroupError::UnhandledError(msg),
            other_error => GroupError::UserError(other_error),
        }
    }
}

impl error::Error for GroupError {}
