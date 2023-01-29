use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum TypeError {
    ResponseError(String),
    UnhandledError(String),
    RuleViolation(String, String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::ResponseError(msg) => write!(f, "{msg}"),
            TypeError::UnhandledError(msg) => write!(f, "Unhandled error: {msg}"),
            TypeError::RuleViolation(action, msg) => {
                write!(f, "Rule {} error: {}", action, msg.replace("_len", "-len"))
            }
        }
    }
}

impl error::Error for TypeError {}
