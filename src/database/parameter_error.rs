use crate::database::CryptoError;
use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ParameterError {
    InvalidFqnOrJmesPath(String),
    RuleViolation(String),
    RuleError(String, String),
    UnhandledError(String),
    ResponseError(String),
    EvaluationError(String),
    CryptoError(CryptoError),
}

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParameterError::InvalidFqnOrJmesPath(msg) => {
                write!(f, "Invalid FQN or JMES path expression: {}", msg)
            }
            ParameterError::RuleViolation(msg) => {
                write!(f, "Rule violation: {}", msg)
            }
            ParameterError::RuleError(action, msg) => {
                write!(f, "Rule {} error: {}", action, msg.replace("_len", "-len"))
            }
            ParameterError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
            ParameterError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
            ParameterError::EvaluationError(msg) => {
                write!(f, "Evaluation error: {}", msg)
            }
            ParameterError::CryptoError(e) => {
                write!(f, "{}", e.to_string())
            }
        }
    }
}

impl From<CryptoError> for ParameterError {
    fn from(e: CryptoError) -> Self {
        ParameterError::CryptoError(e)
    }
}

impl error::Error for ParameterError {}
