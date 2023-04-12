use crate::data::{Name, Scope};

/// Trait for test resources that have a name and can be created and deleted.
/// Used by the Scope struct to construct/cleanup test data server-side
pub trait TestResource {
    /// Access a reference to the Name of this resource
    fn name(&self) -> &Name;
    /// Create the test resource on the server
    fn create(&self);
    /// Delete the test resource from the server
    fn delete(&self);
}

/// Extends test resource with methods to create scoped test data.
///
/// Only one implementation is provided for all TestResources
pub trait ScopedTestResourceExt
where
    Self: Sized + TestResource,
{
    /// Creates a TestResource and then scopes its lifetime to the lifetime of the returned value.
    /// When the scoped value is dropped in memory the resource is automatically deleted.
    fn scoped(self) -> Scope<Self> {
        Scope::new(self)
    }

    /// Creates a TestResource for the lifetime of the given closure.
    fn with_scope<F, R>(self, scope_func: F) -> R
    where
        F: FnOnce(Scope<Self>) -> R,
    {
        scope_func(self.scoped())
    }
}

impl<R> ScopedTestResourceExt for R where R: Sized + TestResource {}
