mod project;

pub use project::*;

use commandspec::CommandArg;
use std::{marker::PhantomData, ops::Deref};
use uuid::Uuid;

/// Trait for name constructors.
/// This is used to create blanket implementations from CreateName and DeleteName
pub trait NameConstructors {
    fn new<S: Into<String>>(name: S) -> Self;
    fn uuid() -> Self;
    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self;
}

/// Trait for creating a name via cli commands. Used by ScopedName to initialize the name
/// in CloudTruth.
pub trait CreateName {
    fn create_name(name: &Name);
}

/// Trait for deleting a name via cli commands. Used by ScopedName to drop the name
/// in CloudTruth.
pub trait DeleteName {
    fn delete_name(name: &Name);
}

/// Blanket implementation for names that implement the create and delete traits
impl<T> NameConstructors for T
where
    T: CreateName + DeleteName + From<ScopedName<T>>,
{
    fn new<S: Into<String>>(name: S) -> Self {
        Self::from(ScopedName::new(name.into()))
    }
    fn uuid() -> Self {
        Self::from(ScopedName::uuid())
    }
    fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self::from(ScopedName::uuid_with_prefix(prefix))
    }
}

/// A newtype wrapper around String representing a generic CloudTruth entity name.
/// Used as a base for other name types.
#[derive(Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

impl NameConstructors for Name {
    fn new<S: Into<String>>(name: S) -> Self {
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
    fn from(value: String) -> Self {
        Self::new(value)
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

/// A generic CloudTruth entity name scoped via Rust borrow checker.
/// Used to implement the more specific scoped structs (ex: ScopedProject)
///
/// T must implement DeleteName, which is used via the Drop implementation
/// to delete the entity when the ScopedName is dropped.
#[derive(Display)]
#[display(fmt = "{}", name)]
pub struct ScopedName<T>
where
    T: DeleteName,
{
    name: Name,
    _phantom: PhantomData<T>,
}

/// Constructors for ScopedName
impl<T> NameConstructors for ScopedName<T>
where
    T: CreateName + DeleteName,
{
    ///Generate custom name
    fn new<S: Into<String>>(name: S) -> Self {
        let name = Name::new(name.into());
        T::create_name(&name);
        ScopedName {
            name,
            _phantom: PhantomData,
        }
    }

    ///Generate new name with UUID
    fn uuid() -> Self {
        ScopedName::new(Name::uuid())
    }

    fn uuid_with_prefix<S>(prefix: S) -> Self
    where
        S: AsRef<str>,
    {
        ScopedName::new(Name::uuid_with_prefix(prefix))
    }
}

impl<T> ScopedName<T>
where
    T: DeleteName,
{
    /// Get a reference to the inner Name for this ScopedName
    pub fn name(&self) -> &Name {
        &self.name
    }
}

/// Auto derefs to underlying Name reference for convenience.
impl<T> Deref for ScopedName<T>
where
    T: DeleteName,
{
    type Target = Name;
    fn deref(&self) -> &Self::Target {
        self.name()
    }
}

impl<T> From<ScopedName<T>> for CommandArg
where
    T: DeleteName,
{
    fn from(scope: ScopedName<T>) -> Self {
        scope.name.clone().into()
    }
}

impl<T> From<&ScopedName<T>> for CommandArg
where
    T: DeleteName,
{
    fn from(scope: &ScopedName<T>) -> Self {
        scope.name().into()
    }
}

impl<T> From<&&ScopedName<T>> for CommandArg
where
    T: DeleteName,
{
    fn from(scope: &&ScopedName<T>) -> Self {
        scope.name().into()
    }
}

/// When ScopedName is dropped, the associated DeleteName::delete_name function of T
/// is called. This is where all cleanup actions occur for scoped test names.
impl<T> Drop for ScopedName<T>
where
    T: DeleteName,
{
    fn drop(&mut self) {
        T::delete_name(&self.name)
    }
}
