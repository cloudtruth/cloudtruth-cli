use crate::command;
use command::Command;

use super::{Name, TestResource};

#[derive(Display, AsRef)]
#[as_ref(forward)]
pub struct ProjectName(Name);

impl TestResource for ProjectName {
    fn name(&self) -> &Name {
        &self.0
    }
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

impl From<&ProjectName> for String {
    fn from(val: &ProjectName) -> Self {
        val.name().to_string()
    }
}
