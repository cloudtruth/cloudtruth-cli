use std::ops::{Deref, DerefMut};

use crate::{
    data::{DeleteTestResource, Name, NameConstructors, TestResource},
    sigint_handler::{SigintHandler, SigintResourceHandle},
};

/// A generic CloudTruth entity scoped via Rust borrow checker.
///
/// Scoped entities are automatically deleted by the Scope's Drop
/// implementation when the value leaves scope.
///
/// Inner type must be a type that implements TestResource  
#[derive(Display)]
#[display(fmt = "{}", resource)]
pub struct Scope<R>
where
    R: TestResource,
{
    resource: R,
    sigint_handle: SigintResourceHandle,
}

impl<R> Scope<R>
where
    R: TestResource,
{
    /// Wrap a TestResource in a scope.
    pub fn new(resource: R) -> Self {
        let sigint_handle = SigintHandler::get_instance()
            .lock()
            .unwrap()
            .register_resource(&resource);
        Scope {
            resource,
            sigint_handle,
        }
    }
}

/// Construct new scoped test resources from a name
impl<R> NameConstructors for Scope<R>
where
    R: TestResource + NameConstructors,
{
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(R::from_name(name))
    }
}

impl<R> Drop for Scope<R>
where
    R: TestResource,
{
    fn drop(&mut self) {
        self.resource.delete()
    }
}

/// Auto derefs to underlying Name reference for convenience.
impl<R> Deref for Scope<R>
where
    R: TestResource,
{
    type Target = R;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

/// Auto derefs to underlying Name reference for convenience.
impl<R> DerefMut for Scope<R>
where
    R: TestResource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
