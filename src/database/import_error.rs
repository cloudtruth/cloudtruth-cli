use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ImportError {
    UnhandledError(String),
    ResponseError(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {msg}")
            }
            ImportError::ResponseError(msg) => {
                write!(f, "{msg}")
            }
        }
    }
}

impl error::Error for ImportError {}
