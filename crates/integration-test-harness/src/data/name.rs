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
pub trait NameConstructors
where
    Self: Sized,
{
    /// Construct a test resource from an existing name.
    fn from_name<N: Into<Name>>(name: N) -> Self;

    /// Construct a new name exactly from a given String.
    fn from_string<S: Into<String>>(string: S) -> Self {
        Self::from_name(Name::from_string(string))
    }
    /// Construct a new name that's automatically generated
    fn generated() -> Self {
        Self::from_name(Name::generated())
    }
    /// Construct a name that's automatically generated with a static prefix
    fn with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self::from_name(Name::with_prefix(prefix))
    }
}

/// Name constructors
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

impl AsRef<OsStr> for Name {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}
