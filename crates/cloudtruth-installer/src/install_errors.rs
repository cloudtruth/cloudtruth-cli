use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::io::Error;
use std::str::Utf8Error;

#[derive(Clone, Debug)]
pub enum InstallError {
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    FailedToRunInstall(String),
    InstallFailed(String),
    Filesystem(String),
    Conversion(String),
}

impl error::Error for InstallError {}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            InstallError::FailedToRunInstall(msg) => {
                write!(f, "Failed to run install script: {msg}")
            }
            InstallError::InstallFailed(msg) => write!(f, "Installation failed: {msg}",),
            InstallError::Filesystem(msg) => write!(f, "File error: {msg}"),
            InstallError::Conversion(msg) => write!(f, "File error: {msg}"),
        }
    }
}

impl From<std::io::Error> for InstallError {
    fn from(err: Error) -> Self {
        Self::Filesystem(err.to_string())
    }
}

impl From<std::str::Utf8Error> for InstallError {
    fn from(err: Utf8Error) -> Self {
        Self::Conversion(err.to_string())
    }
}
