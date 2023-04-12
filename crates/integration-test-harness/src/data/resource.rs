use super::Name;

/// Trait for test resources that have a name and can be created and deleted. Used by the Scoped type to initialize
/// test data in CloudTruth
pub trait TestResource {
    /// Access a reference to the Name of this resource
    fn name(&self) -> &Name;
    /// Create the test resource on the server
    fn create(&self);
    /// Delete the test resource from the server
    fn delete(&self);
}
