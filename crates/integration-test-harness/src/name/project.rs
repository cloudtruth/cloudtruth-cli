use command::Command;

use super::{CreateName, DeleteName, Name, ScopedName};
use crate::command;

#[derive(Display)]
pub struct ScopedProject(ScopedName<ScopedProject>);

impl CreateName for ScopedProject {
    fn create_name(name: &Name) {
        Command::cargo_bin("cloudtruth")
            .unwrap()
            .args(["projects", "set", name.as_str()]);
    }
}

impl DeleteName for ScopedProject {
    fn delete_name(name: &Name) {
        Command::cargo_bin("cloudtruth").unwrap().args([
            "projects",
            "delete",
            "--confirm",
            name.as_str(),
        ]);
    }
}

impl From<ScopedName<ScopedProject>> for ScopedProject {
    fn from(value: ScopedName<ScopedProject>) -> Self {
        Self(value)
    }
}
