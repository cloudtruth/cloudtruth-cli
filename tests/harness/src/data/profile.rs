use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Environment, Name, NameConstructors, Project, Scope, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Profile<'a, 'prof, 'proj, 'e> {
    name: Name,
    api_key: Option<&'a str>,
    source: Option<&'prof Name>,
    project: Option<&'proj Name>,
    env: Option<&'e Name>,
}

impl<'a, 'prof, 'proj, 'e> Profile<'a, 'prof, 'proj, 'e> {
    fn new(name: Name) -> Self {
        Self {
            name,
            api_key: None,
            source: None,
            project: None,
            env: None,
        }
    }

    /// Set the projects description.
    pub fn api_key<A: AsRef<str> + ?Sized>(mut self, api_key: &'a A) -> Self {
        self.api_key = Some(api_key.as_ref());
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

    /// Set the profiles default environment
    pub fn env(mut self, env: &'e Environment) -> Self {
        self.env = Some(env.name());
        self
    }
}

impl<'a, 'prof, 'proj, 'e> NameConstructors for Profile<'a, 'prof, 'proj, 'e> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into())
    }
}

impl<'a, 'prof, 'proj, 'e> TestResource for Profile<'a, 'prof, 'proj, 'e> {
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
        if let Some(env) = self.env {
            cmd.args(["-e", env.as_str()]);
        }
        cmd.assert()
            .success()
            .stdout(contains!("Created profile '{self}'"));
        Scope::new(self)
    }

    fn delete_cmd(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(cli_bin_path("cloudtruth"));
        cmd.args([
            "configuration",
            "profile",
            "delete",
            "--confirm",
            self.name.as_str(),
        ]);
        cmd
    }
}
