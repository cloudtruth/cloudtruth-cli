use serde_json::Value;
use std::{env, ffi::OsStr};
use uuid::Uuid;

const JOB_ID: &str = "JOB_ID";
const NEXTEST_RUN_ID: &str = "NEXTEST_RUN_ID";

/// Use JOB_ID or NEXTEST_RUN_ID if available, otherwise generate random
fn generated_test_id() -> String {
    env::var(JOB_ID)
        .or_else(|_| env::var(NEXTEST_RUN_ID))
        .unwrap_or_else(|_| Uuid::new_v4().to_string())
}

/// A newtype wrapper around String representing a generic CloudTruth entity name.
/// Used as a base for other name types.
#[derive(Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

/// Trait for name constructors.
///
/// Default implementations for all other methods are provided as long as from_name is implemented.
pub trait NameConstructors: Sized {
    /// Constructs a value from an existing name.
    fn from_name<N: Into<Name>>(name: N) -> Self;

    /// Construct a value exactly from a given String.
    fn from_string<S: Into<String>>(string: S) -> Self {
        Self::from_name(Name::from_string(string))
    }
    /// Construct a value that's given an automatically generated a name
    fn generated() -> Self {
        Self::from_name(Name::generated())
    }
    /// Construct a value that's given an automatically generated name with a static prefix
    fn with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self::from_name(Name::with_prefix(prefix))
    }
}

/// Name constructors for the base Name type
impl NameConstructors for Name {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        name.into()
    }
    fn from_string<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }

    fn generated() -> Self {
        Self(format!("test-{}", generated_test_id()))
    }

    fn with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self(format!("{}-test-{}", prefix.as_ref(), generated_test_id()))
    }
}

impl Name {
    /// Represent a Name as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Name {
    /// Convert a String into a Name
    fn from(string: String) -> Self {
        Self::from_string(string)
    }
}

impl From<Name> for String {
    /// Convert a Name into a String
    fn from(name: Name) -> Self {
        name.0
    }
}

impl From<&Name> for String {
    /// Convert a reference to a Name to a String by cloning it.
    /// Needed for easy use of predicate functions.
    fn from(name: &Name) -> Self {
        name.0.clone()
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for Name {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl From<Name> for Value {
    fn from(val: Name) -> Self {
        val.0.into()
    }
}

impl From<&Name> for Value {
    fn from(val: &Name) -> Self {
        val.as_str().into()
    }
}
