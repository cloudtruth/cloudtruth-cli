use command::Command;
use std::ops::Deref;

use super::name::Name;
use crate::{command, scopes::ScopedName};

#[derive(Display)]
pub struct ScopedProject(ScopedName);

impl ScopedProject {
    pub fn new<N: Into<Name>>(name: N) -> Self {
        Self(ScopedName::new(
            name.into(),
            Self::create_cmd,
            Self::delete_cmd,
        ))
    }

    ///Generate new name with UUID
    pub fn uuid() -> Self {
        Self(ScopedName::uuid(Self::create_cmd, Self::delete_cmd))
    }

    pub fn uuid_with_prefix<S: AsRef<str>>(prefix: S) -> Self {
        Self(ScopedName::uuid_with_prefix(
            prefix,
            Self::create_cmd,
            Self::delete_cmd,
        ))
    }

    fn create_cmd(name: &Name) {
        Command::cargo_bin("cloudtruth")
            .unwrap()
            .args(["projects", "set", name.as_str()]);
    }

    fn delete_cmd(name: &Name) {
        Command::cargo_bin("cloudtruth").unwrap().args([
            "projects",
            "delete",
            "--confirm",
            name.as_str(),
        ]);
    }
}

impl Deref for ScopedProject {
    type Target = ScopedName;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
