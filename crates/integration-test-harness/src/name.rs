mod project;
mod scope;

pub use project::*;
pub use scope::*;

use commandspec::CommandArg;
use uuid::Uuid;

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
        Self(Uuid::new_v4().to_string())
    }

    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self(format!("{}-{}", prefix.as_ref(), Uuid::new_v4()))
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

impl From<Name> for CommandArg {
    fn from(name: Name) -> Self {
        CommandArg::Literal(name.0)
    }
}

impl From<&Name> for CommandArg {
    fn from(name: &Name) -> Self {
        CommandArg::Literal(name.to_string())
    }
}

impl From<&&Name> for CommandArg {
    fn from(name: &&Name) -> Self {
        CommandArg::Literal(name.to_string())
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

/// Trait for test resources that have a name and can be created and deleted. Used by the Scoped type to initialize
/// test data in CloudTruth
pub trait TestResource {
    /// Access a reference to the Name of this resource
    fn name(&self) -> &Name;
    /// Construct a test resource from a name
    fn from_name<N: Into<Name>>(name: N) -> Self;
    /// Create the test resource on the server
    fn create(&self);
    /// Delete the test resource from the server
    fn delete(&self);
}

/// Blanket implementation of NameConstructors for types that implement TestResource
impl<Resource> NameConstructors for Resource
where
    Resource: TestResource,
{
    fn from_string<S: Into<String>>(string: S) -> Self {
        Self::from_name(Name::from_string(string))
    }
    fn uuid() -> Self {
        Self::from_name(Name::uuid())
    }
    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self::from_name(Name::uuid_with_prefix(prefix))
    }
}
