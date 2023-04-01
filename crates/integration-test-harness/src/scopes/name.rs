use commandspec::CommandArg;
use std::{borrow::Cow, ops::Deref};
use uuid::Uuid;

fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// A generic CloudTruth entity name
#[derive(Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

impl Name {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }

    pub fn uuid() -> Self {
        Self(uuid())
    }

    pub fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self(format!("{}-{}", prefix.as_ref(), uuid()))
    }

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

impl From<Name> for Cow<'static, str> {
    fn from(name: Name) -> Self {
        Cow::Owned(name.to_string())
    }
}

impl From<&'static Name> for Cow<'static, str> {
    fn from(name: &'static Name) -> Self {
        Cow::Borrowed(&name.0)
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
///
/// This is used to implement the more specific scope structs (ex: ProjectScope, EnvScope)
#[derive(Display)]
#[display(fmt = "{}", name)]
pub struct ScopedName {
    name: Name,
    delete_fn: Box<dyn FnMut(&Name)>,
}

impl ScopedName {
    ///Generate custom name
    pub fn new<N, FC, FD>(name: N, create_fn: FC, delete_fn: FD) -> Self
    where
        N: Into<Name>,
        FC: FnOnce(&Name),
        FD: FnMut(&Name) + 'static,
    {
        let name = name.into();
        create_fn(&name);
        ScopedName {
            name,
            delete_fn: Box::new(delete_fn),
        }
    }

    ///Generate new name with UUID
    pub fn uuid<FC, FD>(create_fn: FC, delete_fn: FD) -> Self
    where
        FC: FnOnce(&Name),
        FD: FnMut(&Name) + 'static,
    {
        ScopedName::new(Name::uuid(), create_fn, delete_fn)
    }

    pub fn uuid_with_prefix<S, FC, FD>(prefix: S, create_fn: FC, delete_fn: FD) -> Self
    where
        S: AsRef<str>,
        FC: FnOnce(&Name),
        FD: FnMut(&Name) + 'static,
    {
        ScopedName::new(Name::uuid_with_prefix(prefix), create_fn, delete_fn)
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    fn run_delete(&mut self) {
        (self.delete_fn)(&self.name);
    }
}

impl Deref for ScopedName {
    type Target = Name;
    fn deref(&self) -> &Self::Target {
        self.name()
    }
}

impl From<ScopedName> for CommandArg {
    fn from(scope: ScopedName) -> Self {
        scope.name.clone().into()
    }
}

impl From<&ScopedName> for CommandArg {
    fn from(scope: &ScopedName) -> Self {
        scope.name().into()
    }
}

impl From<&&ScopedName> for CommandArg {
    fn from(scope: &&ScopedName) -> Self {
        scope.name().into()
    }
}

impl Drop for ScopedName {
    fn drop(&mut self) {
        self.run_delete()
    }
}
