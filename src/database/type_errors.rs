use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum TypeError {
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::ResponseError(msg) => write!(f, "{}", msg),
            TypeError::UnhandledError(msg) => write!(f, "Unhandled error: {}", msg),
        }
    }
}

impl error::Error for TypeError {}
