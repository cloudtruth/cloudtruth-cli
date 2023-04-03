use crate::command;
use command::Command;

use super::{HasName, Name, TestResource};

#[derive(Display)]
pub struct ProjectName(Name);

impl HasName for ProjectName {
    fn name(&self) -> &Name {
        &self.0
    }
}

impl TestResource for ProjectName {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self(name.into())
    }

    fn create(&self) {
        Command::cargo_bin("cloudtruth")
            .args(["projects", "set", self.0.as_str()])
            .assert()
            .success();
    }

    fn delete(&self) {
        Command::cargo_bin("cloudtruth")
            .args(["projects", "delete", "--confirm", self.0.as_str()])
            .assert()
            .success();
    }
}
