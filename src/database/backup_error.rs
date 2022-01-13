use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum BackupError {
    UnhandledError(String),
    ResponseError(String),
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BackupError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
            BackupError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl error::Error for BackupError {}
