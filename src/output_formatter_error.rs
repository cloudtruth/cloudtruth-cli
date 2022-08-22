use color_eyre::eyre::ErrReport;
use csv::Error as CsvError;
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::fmt;
use std::fmt::Formatter;
use std::io::Error as IoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputFormatterError {
    UnhandledFormat(String),
    CsvError(String),
    JsonError(String),
    WriteError(String),
    YamlError(String),
}

impl fmt::Display for OutputFormatterError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnhandledFormat(format) => {
                write!(f, "Unhandled format: {}", format)
            }
            Self::CsvError(details) => {
                write!(f, "CSV error: {}", details)
            }
            Self::JsonError(details) => {
                write!(f, "JSON error: {}", details)
            }
            Self::WriteError(details) => {
                write!(f, "IO error: {}:", details)
            }
            Self::YamlError(details) => {
                write!(f, "YAML error: {}", details)
            }
        }
    }
}

// NOTE: CsvError does not implement the Clone trait, so just pass through a String
impl From<CsvError> for OutputFormatterError {
    fn from(err: CsvError) -> Self {
        Self::CsvError(err.to_string())
    }
}

// NOTE: IoError does not implement the Clone trait, so just pass through a String
impl From<IoError> for OutputFormatterError {
    fn from(err: IoError) -> Self {
        Self::WriteError(err.to_string())
    }
}

// NOTE: JsonError does not implement the Clone trait, so just pass through a String
impl From<JsonError> for OutputFormatterError {
    fn from(err: JsonError) -> Self {
        Self::JsonError(err.to_string())
    }
}

// NOTE: YamlError does not implement the Clone trait, so just pass through a String
impl From<YamlError> for OutputFormatterError {
    fn from(err: YamlError) -> Self {
        Self::YamlError(err.to_string())
    }
}

impl From<OutputFormatterError> for ErrReport {
    fn from(err: OutputFormatterError) -> Self {
        ErrReport::msg(err.to_string())
    }
}
