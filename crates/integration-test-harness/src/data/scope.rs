use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
};

use crate::data::{Name, NameConstructors, TestResource};

/// A generic CloudTruth entity scoped via Rust borrow checker.
///
/// Scoped entities are automatically created when the Scope is created,
/// and automatically deleted by the Scope's Drop implementation when the
/// value leaves scope.
///
/// Inner type must be a type that implements TestResource  
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
    /// Create a scope from a given TestResource. This calls the resources create() method
    pub fn new(mut resource: R) -> Self {
        resource.create();
        Scope { resource }
    }
}

/// Constructors for Scope. All constructors call the TestResource create() method
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

impl<'a, R> From<&'a Scope<R>> for String
where
    R: TestResource,
{
    /// Convert a Scope reference to a String by cloning. Needed for easy use of predicate functions
    fn from(value: &'a Scope<R>) -> Self {
        value.name().into()
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
