use crate::database::{EnvironmentError, ProjectError};
use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ResolveError {
    ResolutionNotFound(Vec<String>),
    EnvironmentError(EnvironmentError),
    ProjectError(ProjectError),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::ResolutionNotFound(strs) => {
                let sep = "\n  ";
                write!(f, "Failed to resolve:{}{}", sep, strs.join(sep))
            }
            ResolveError::EnvironmentError(e) => {
                write!(f, "Environment error: {}", e)
            }
            ResolveError::ProjectError(e) => {
                write!(f, "Project error: {}", e)
            }
        }
    }
}

impl error::Error for ResolveError {}

impl From<EnvironmentError> for ResolveError {
    fn from(e: EnvironmentError) -> Self {
        Self::EnvironmentError(e)
    }
}

impl From<ProjectError> for ResolveError {
    fn from(e: ProjectError) -> Self {
        Self::ProjectError(e)
    }
}
