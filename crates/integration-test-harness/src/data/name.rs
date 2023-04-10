use std::env;
use uuid::Uuid;

const NEXTEST_RUN_ID: &str = "NEXTEST_RUN_ID";

/// Use NEXTEST_RUN_ID if available, otherwise generate random
fn uuid() -> String {
    env::var(NEXTEST_RUN_ID).unwrap_or_else(|_| Uuid::new_v4().to_string())
}

/// A newtype wrapper around String representing a generic CloudTruth entity name.
/// Used as a base for other name types.
#[derive(Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

/// Trait for name constructors.
///
/// A blanket implementation is provided for any types that implement
/// TestResource so that newtype wrappers for specific entity
/// types only need to implement those trait methods to have standardized constructor functions
/// and the auto-delete behavior via the Drop implementation for Scoped.
///
/// See src/name/project.rs for an example of how this is done.
pub trait NameConstructors {
    /// Construct a new name directly from a String.
    fn from_string<S: Into<String>>(name: S) -> Self;
    /// Construct a new name from a v4 UUID.
    fn uuid() -> Self;
    /// Generate new name with a v4 UUID and a fixed prefix.
    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self;
}

/// Name constructors
impl NameConstructors for Name {
    fn from_string<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }

    fn uuid() -> Self {
        Self(uuid())
    }

    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self(format!("{}-{}", prefix.as_ref(), uuid()))
    }
}

impl Name {
    /// Represent name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Name {
    fn from(string: String) -> Self {
        Self::from_string(string)
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.0
    }
}

impl From<&Name> for String {
    fn from(name: &Name) -> Self {
        name.0.clone()
    }
}

impl<'a> From<&'a Name> for &'a String {
    fn from(name: &'a Name) -> Self {
        &name.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
