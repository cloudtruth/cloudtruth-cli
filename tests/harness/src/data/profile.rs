use std::borrow::Cow;

use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, TestResource},
    prelude::Project,
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Profile<'a, 'prof, 'proj> {
    name: Name,
    api_key: Option<Cow<'a, str>>,
    source: Option<&'prof Name>,
    project: Option<&'proj Name>,
}

impl<'a, 'prof, 'proj> Profile<'a, 'prof, 'proj> {
    fn new(name: Name) -> Self {
        Self {
            name,
            api_key: None,
            source: None,
            project: None,
        }
    }

    /// Set the projects description.
    pub fn api_key<A: Into<Cow<'a, str>>>(mut self, api_key: A) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the profiles source (i.e. parent) profile
    pub fn source(mut self, source: &'prof Profile) -> Self {
        self.source = Some(source.name());
        self
    }

    /// Set the profiles default project
    pub fn project(mut self, project: &'proj Project) -> Self {
        self.project = Some(project.name());
        self
    }
}

impl<'a, 'prof, 'proj> NameConstructors for Profile<'a, 'prof, 'proj> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into())
    }
}

impl<'a, 'prof, 'proj> TestResource for Profile<'a, 'prof, 'proj> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(self) -> Scope<Self> {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["configuration", "profile", "set", self.name.as_str()]);
        if let Some(api_key) = &self.api_key {
            cmd.args(["-k", api_key]);
        }
        if let Some(source) = self.source {
            cmd.args(["-s", source.as_str()]);
        }
        if let Some(project) = self.project {
            cmd.args(["-p", project.as_str()]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created profile '{self}'"));
        Scope::new(self)
    }

    fn delete(&mut self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "configuration",
                "profile",
                "delete",
                "--confirm",
                self.name.as_str(),
            ])
            .assert()
            .success();
    }
}
