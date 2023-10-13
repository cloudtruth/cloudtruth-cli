use std::process::Stdio;

use crate::{
    data::{Name, Scope},
    util::retry_cmd_with_backoff,
};

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

    /// Returns a command that will delete the test resource from the server when executed
    fn delete_cmd(&self) -> std::process::Command;

    /// Creates a TestResource for the lifetime of the given closure.
    fn with_scope<F, R>(self, scope_func: F) -> R
    where
        F: FnOnce(Scope<Self>) -> R,
    {
        scope_func(self.create())
    }
}

// An object-safe trait that allows creation of trait objects to delete TestResources
pub trait DeleteTestResource {
    fn delete(&self);
}

impl<R: TestResource> DeleteTestResource for R {
    fn delete(&self) {
        let mut cmd = self.delete_cmd();
        cmd.stdout(Stdio::null());
        let _res = retry_cmd_with_backoff(&mut cmd);
    }
}
