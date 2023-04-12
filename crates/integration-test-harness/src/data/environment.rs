use super::{Name, NameConstructors, Scope, Scoped, TestResource};
use crate::{
    command::{cli_bin_path, Command},
    contains,
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
        Self::new(name.into(), None, None)
    }
}

impl<'d, 'p> TestResource for Environment<'d, 'p> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(&self) {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["environments", "set", self.name.as_str()]);
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
            .args(["environments", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success();
    }
}

impl<'d, 'p> From<&Environment<'d, 'p>> for String {
    fn from(name: &Environment) -> Self {
        name.name().into()
    }
}

impl<'d, 'p> From<Environment<'d, 'p>> for String {
    fn from(environment: Environment) -> Self {
        environment.name.into()
    }
}

pub struct EnvironmentBuilder<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> NameConstructors for EnvironmentBuilder<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        EnvironmentBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }
}

impl<'d, 'p> EnvironmentBuilder<'d, 'p> {
    pub fn new<N: Into<Name>>(name: N) -> Self {
        EnvironmentBuilder {
            name: name.into(),
            description: None,
            parent: None,
        }
    }

    pub fn description(mut self, description: &'d str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn parent(mut self, parent: &'p Environment) -> Self {
        self.parent = Some(&parent.name);
        self
    }

    pub fn build_scoped(self) -> Scope<Environment<'d, 'p>> {
        self.build().scoped()
    }

    pub fn build(self) -> Environment<'d, 'p> {
        Environment::new(self.name, self.description, self.parent)
    }
}
