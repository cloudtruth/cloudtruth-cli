use std::ops::Deref;

use super::{HasName, Name, NameConstructors, TestResource};

/// A generic CloudTruth entity name scoped via Rust borrow checker.
/// Used to implement the more specific scoped structs (ex: ScopedProject)
///
/// T must implement DeleteName, which is used via the Drop implementation
/// to delete the entity when the ScopedName is dropped.
#[derive(Display)]
#[display(fmt = "{}", resource)]
pub struct Scope<Resource>
where
    Resource: TestResource,
{
    resource: Resource,
}

impl<Resource> Scope<Resource>
where
    Resource: TestResource,
{
    pub fn new(resource: Resource) -> Self {
        resource.create();
        Scope { resource }
    }
}

/// Constructors for Scoped
impl<Resource> NameConstructors for Scope<Resource>
where
    Resource: TestResource,
{
    ///Generate custom name
    fn from_string<S: Into<String>>(string: S) -> Self {
        Self::new(Resource::from_string(string.into()))
    }

    fn uuid() -> Self {
        Scope::new(Resource::from_name(Name::uuid()))
    }

    fn uuid_with_prefix<S>(prefix: S) -> Self
    where
        S: AsRef<str>,
    {
        Scope::new(Resource::from_name(Name::uuid_with_prefix(prefix)))
    }
}

impl<Resource> HasName for Scope<Resource>
where
    Resource: TestResource + HasName,
{
    /// Get a reference to the inner Name for this Scoped resource
    fn name(&self) -> &Name {
        self.resource.name()
    }
}

/// When ScopedName is dropped, the associated DeleteName::delete_name function of T
/// is called. This is where all cleanup actions occur for scoped test names.
impl<Resource> Drop for Scope<Resource>
where
    Resource: TestResource,
{
    fn drop(&mut self) {
        self.resource.delete()
    }
}

/// Auto derefs to underlying Name reference for convenience.
impl<Resource> Deref for Scope<Resource>
where
    Resource: TestResource,
{
    type Target = Resource;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

/// A trait to take a test resource and scope it via the Rust borrow checker.
///
/// Blanket implementation provided for TestResource implementors
pub trait Scoped
where
    Self: Sized + TestResource,
{
    fn scoped(self) -> Scope<Self>;
}

impl<N> Scoped for N
where
    N: Sized + TestResource,
{
    fn scoped(self) -> Scope<Self> {
        Scope { resource: self }
    }
}
