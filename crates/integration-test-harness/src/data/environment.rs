use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, ScopedTestResourceExt, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Environment<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

pub type ScopedEnvironment<'d, 'p> = Scope<Environment<'d, 'p>>;

impl<'d, 'p> Environment<'d, 'p> {
    fn new<N: Into<Name>>(name: N, description: Option<&'d str>, parent: Option<&'p Name>) -> Self {
        Self {
            name: name.into(),
            description,
            parent,
        }
    }

    pub fn rename<N: Into<Name>>(&mut self, name: N) -> &mut Self {
        let name = name.into();
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "environments",
                "set",
                self.name.as_str(),
                "--rename",
                name.as_str(),
            ])
            .assert()
            .success()
            .stdout(contains!("Updated environment '{name}'"));
        self.name = name;
        self
    }
}

impl<'d, 'p> NameConstructors for Environment<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name, None, None)
    }
}

impl<'d, 'p> TestResource for Environment<'d, 'p> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(&mut self) {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["environments", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        if let Some(parent) = self.parent {
            cmd.args(["--parent", parent.as_ref()]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created environment '{self}'"));
    }

    fn delete(&mut self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args(["environments", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success();
    }
}

impl<'d, 'p> From<&Environment<'d, 'p>> for String {
    /// Convert an Environment reference to a String by cloning its name.
    /// Needed for easy use of predicate functions.
    fn from(name: &Environment) -> Self {
        name.name().into()
    }
}

impl<'d, 'p> From<Environment<'d, 'p>> for String {
    /// Convert an Environment to a String of its name.
    fn from(environment: Environment) -> Self {
        environment.name.into()
    }
}

/// For more complex Environment data, use an EnvironmentBuilder to fill in the values and then call
/// build() or build_scoped() to create the Environment or Scope<Environment>, respectively.
pub struct EnvironmentBuilder<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> NameConstructors for EnvironmentBuilder<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        EnvironmentBuilder::new(name)
    }
}

impl<'d, 'p> EnvironmentBuilder<'d, 'p> {
    /// Create a EnvironmentBuilder from a name.
    pub fn new<N: Into<Name>>(name: N) -> Self {
        EnvironmentBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }

    /// Set the environments description.
    pub fn description<D: AsRef<str> + ?Sized>(mut self, description: &'d D) -> Self {
        self.description = Some(description.as_ref());
        self
    }

    /// Set the environments parent.
    pub fn parent(mut self, parent: &'p Environment) -> Self {
        self.parent = Some(&parent.name);
        self
    }

    /// Build a Scope<Environment>. Equivalent to calling .build().scoped().
    /// Because creating a Scope around a Environment creates the environment on the server,
    /// this method does the same.
    pub fn build_scoped(self) -> Scope<Environment<'d, 'p>> {
        self.build().scoped()
    }

    /// Build an Environment. Does not automatically create the Environment on the server.
    pub fn build(self) -> Environment<'d, 'p> {
        Environment::new(self.name, self.description, self.parent)
    }
}
