use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct ParamType<'d, 'p> {
    name: Name,
    description: Option<&'d str>,
    parent: Option<&'p Name>,
}

impl<'d, 'p> ParamType<'d, 'p> {
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
                "types",
                "set",
                self.name.as_str(),
                "--rename",
                name.as_str(),
            ])
            .assert()
            .success()
            .stdout(contains!("Updated parameter type '{name}'"));
        self.name = name;
        self
    }

    /// Set the parameter types description.
    pub fn description<D: AsRef<str> + ?Sized>(mut self, description: &'d D) -> Self {
        self.description = Some(description.as_ref());
        self
    }

    /// Set the parameter types parent.
    pub fn parent(mut self, parent: &'p ParamType) -> Self {
        self.parent = Some(&parent.name);
        self
    }
}

impl<'d, 'p> NameConstructors for ParamType<'d, 'p> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into(), None, None)
    }
}

impl<'d, 'p> TestResource for ParamType<'d, 'p> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(self) -> Scope<Self> {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["types", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        if let Some(parent) = self.parent {
            cmd.args(["--parent", parent.as_ref()]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created parameter type '{self}'"));
        Scope::new(self)
    }

    fn delete_cmd(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["types", "delete", "--confirm", self.name.as_str()]);
        cmd
    }
}
