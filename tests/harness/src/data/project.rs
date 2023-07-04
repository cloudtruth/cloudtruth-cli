use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Project<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> Project<'d, 'p> {
    fn new(name: Name, description: Option<&'d str>, parent: Option<&'p Name>) -> Self {
        Self {
            name,
            description,
            parent,
        }
    }

    pub fn rename<N: Into<Name>>(&mut self, name: N) -> &mut Self {
        let name = name.into();
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "projects",
                "set",
                self.name.as_str(),
                "--rename",
                name.as_str(),
            ])
            .assert()
            .success()
            .stdout(contains!("Updated project '{name}'"));
        self.name = name;
        self
    }

    pub fn copy<N: Into<Name>>(&self, name: N) -> Scope<Self> {
        let name = name.into();
        Command::new(cli_bin_path("cloudtruth"))
            .args(["projects", "copy", self.name.as_str(), name.as_str()])
            .assert()
            .success()
            .stdout(contains!(
                "Copied project '{src_name}' to '{dest_name}",
                src_name = self.name,
                dest_name = name
            ));
        Scope::new(Project::new(name, self.description, self.parent))
    }

    /// Set the projects description.
    pub fn description<D: AsRef<str> + ?Sized>(mut self, description: &'d D) -> Self {
        self.description = Some(description.as_ref());
        self
    }

    /// Set the projects parent.
    pub fn parent(mut self, parent: &'p Project) -> Self {
        self.parent = Some(parent.name());
        self
    }
}

impl<'d, 'p> NameConstructors for Project<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into(), None, None)
    }
}

impl<'d, 'p> TestResource for Project<'d, 'p> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(self) -> Scope<Self> {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["projects", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        if let Some(parent) = self.parent {
            cmd.args(["--parent", parent.as_str()]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created project '{self}'"));
        Scope::new(self)
    }

    fn delete(&mut self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args(["projects", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success();
    }
}
