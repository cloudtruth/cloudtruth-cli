use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
};

use super::{Name, NameConstructors, TestResource};

/// A generic CloudTruth entity name scoped via Rust borrow checker.
/// Used to implement the more specific scoped structs (ex: ScopedProject)
///
/// T must implement DeleteName, which is used via the Drop implementation
/// to delete the entity when the ScopedName is dropped.
#[derive(Display)]
#[display(fmt = "{}", resource)]
pub struct Scope<R>
where
    R: TestResource,
{
    resource: R,
}

impl<R> Scope<R>
where
    R: TestResource,
{
    pub fn new(resource: R) -> Self {
        resource.create();
        Scope { resource }
    }
}

/// Constructors for Scoped
impl<R> NameConstructors for Scope<R>
where
    R: TestResource + NameConstructors,
{
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(R::from_name(name))
    }
}

/// When ScopedName is dropped, the associated DeleteName::delete_name function of T
/// is called. This is where all cleanup actions occur for scoped test names.
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

impl<'a, R> From<&'a Scope<R>> for String
where
    R: TestResource,
{
    fn from(value: &'a Scope<R>) -> Self {
        value.name().into()
    }
}

/// A trait to take a test resource and scope it via the Rust borrow checker.
///
/// Blanket implementation provided for TestResource implementors
pub trait Scoped
where
    Self: Sized + TestResource,
{
    /// Creates a TestResource and then scopes its lifetime to the lifetime of the returned value.
    /// When the scoped value is dropped in memory the resource is automatically deleted.
    fn scoped(self) -> Scope<Self>;

    /// Creates a TestResource for the lifetime of the given closure.
    fn with_scope<F, R>(self, mut scope_func: F) -> R
    where
        F: FnMut(Scope<Self>) -> R,
    {
        scope_func(self.scoped())
    }
}

impl<R> Scoped for R
where
    R: Sized + TestResource,
{
    fn scoped(self) -> Scope<Self> {
        Scope::new(self)
    }
}

impl<R> AsRef<str> for Scope<R>
where
    R: TestResource,
{
    fn as_ref(&self) -> &str {
        self.name().as_ref()
    }
}

impl<R> AsRef<OsStr> for Scope<R>
where
    R: TestResource,
{
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.name().as_str())
    }
}
