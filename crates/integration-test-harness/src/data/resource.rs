use super::{Name, NameConstructors};

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
