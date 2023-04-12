use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, ScopedTestResourceExt, TestResource},
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
    /// Convert a Project reference to a String by cloning its name.
    /// Needed for easy use of predicate functions.
    fn from(name: &Project) -> Self {
        name.name().into()
    }
}

impl<'d, 'p> From<Project<'d, 'p>> for String {
    /// Convert a Project to a String representing its name
    fn from(project: Project) -> Self {
        project.name.into()
    }
}

/// For more complex Project data, use a ProjectBuilder to fill in the values and then call
/// build() or build_scoped() to create the Project or Scope<Project>, respectively.
pub struct ProjectBuilder<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> NameConstructors for ProjectBuilder<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        ProjectBuilder::new(name)
    }
}

impl<'d, 'p> ProjectBuilder<'d, 'p> {
    /// Create a ProjectBuilder from a name.
    pub fn new<N: Into<Name>>(name: N) -> Self {
        ProjectBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }

    /// Set the projects description.
    pub fn description(mut self, description: &'d str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the projects parent.
    pub fn parent(mut self, parent: &'p Project) -> Self {
        self.parent = Some(&parent.name);
        self
    }

    /// Build a Scope<Project>. Equivalent to calling .build().scoped(). Because creating a Scope around a Project creates the project on the server,
    /// this method does the same.
    pub fn build_scoped(self) -> Scope<Project<'d, 'p>> {
        self.build().scoped()
    }

    /// Build a Project. Does not automatically create the Project on the server.
    pub fn build(self) -> Project<'d, 'p> {
        Project::new(self.name, self.description, self.parent)
    }
}
