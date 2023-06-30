use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Group<'d> {
    name: Name,
    description: Option<&'d str>,
}

impl<'d> Group<'d> {
    fn new(name: Name, description: Option<&'d str>) -> Self {
        Self { name, description }
    }

    /// Set the groups description.
    pub fn description<D: AsRef<str> + ?Sized>(mut self, description: &'d D) -> Self {
        self.description = Some(description.as_ref());
        self
    }

    pub fn rename<N: Into<Name>>(&mut self, name: N) -> &mut Self {
        let name = name.into();
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "groups",
                "set",
                self.name.as_str(),
                "--rename",
                name.as_str(),
            ])
            .assert()
            .success()
            .stdout(contains!("Updated group '{name}'"));
        self.name = name;
        self
    }
}

impl<'d> NameConstructors for Group<'d> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into(), None)
    }
}

impl<'d> TestResource for Group<'d> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(self) -> Scope<Self> {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["groups", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created group '{self}'"));
        Scope::new(self)
    }

    fn delete(&mut self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args(["groups", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success()
            .stdout(contains!("Deleted group '{self}'"));
    }
}
