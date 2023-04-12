use super::{Name, NameConstructors, Scope, Scoped, TestResource};
use crate::{
    command::{cli_bin_path, Command},
    contains,
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Project<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

pub type ScopedProject<'d, 'p> = Scope<Project<'d, 'p>>;

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
    fn create(&self) {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["projects", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        if let Some(parent) = self.parent {
            cmd.args(["--parent", parent.as_ref()]);
        }
        cmd.assert().success();
    }

    fn delete(&self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args(["projects", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success();
    }
}

impl<'d, 'p> From<&Project<'d, 'p>> for String {
    fn from(name: &Project) -> Self {
        name.name().into()
    }
}

impl<'d, 'p> From<Project<'d, 'p>> for String {
    fn from(project: Project) -> Self {
        project.name.into()
    }
}

pub struct ProjectBuilder<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> NameConstructors for ProjectBuilder<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        ProjectBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }
}

impl<'d, 'p> ProjectBuilder<'d, 'p> {
    pub fn new<N: Into<Name>>(name: N) -> Self {
        ProjectBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }

    pub fn description(mut self, description: &'d str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn parent(mut self, parent: &'p Project) -> Self {
        self.parent = Some(&parent.name);
        self
    }

    pub fn build_scoped(self) -> Scope<Project<'d, 'p>> {
        self.build().scoped()
    }

    pub fn build(self) -> Project<'d, 'p> {
        Project::new(self.name, self.description, self.parent)
    }
}
