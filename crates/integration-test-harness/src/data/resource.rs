use crate::data::{Name, Scope};

/// Trait for test resources that have a name and can be created and deleted.
/// Used by the Scope struct to construct/cleanup test data server-side
pub trait TestResource: Sized {
    /// Access a reference to the Name of this resource
    fn name(&self) -> &Name;
    /// Get a clone of the test resources Name
    fn to_name(&self) -> Name {
        self.name().clone()
    }
    /// Create the test resource on the server
    fn create(self) -> Scope<Self>;
    /// Delete the test resource from the server
    fn delete(&mut self);

    /// Creates a TestResource for the lifetime of the given closure.
    fn with_scope<F, R>(self, scope_func: F) -> R
    where
        F: FnOnce(Scope<Self>) -> R,
    {
        scope_func(self.create())
    }
}
