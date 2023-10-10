use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct User<'d, 'r> {
    name: Name,
    description: Option<&'d str>,
    role: Option<&'r str>,
    api_key: Option<String>,
}

impl<'d, 'r> User<'d, 'r> {
    fn new(name: Name, description: Option<&'d str>, role: Option<&'r str>) -> Self {
        Self {
            name,
            description,
            role,
            api_key: None,
        }
    }

    // Fetches the API key. Panics if the user has not been created with an API key yet.
    pub fn api_key(&self) -> &str {
        self.api_key
            .as_ref()
            .expect("Service account was not created and does not have an API key")
    }

    /// Set the users description.
    pub fn description<D: AsRef<str> + ?Sized>(mut self, description: &'d D) -> Self {
        self.description = Some(description.as_ref());
        self
    }

    /// Set the users role.
    pub fn role<R: AsRef<str> + ?Sized>(mut self, role: &'r R) -> Self {
        self.role = Some(role.as_ref());
        self
    }
}

impl<'d, 'r> NameConstructors for User<'d, 'r> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name.into(), None, None)
    }
}

impl<'d, 'r> TestResource for User<'d, 'r> {
    fn name(&self) -> &Name {
        &self.name
    }
    fn create(mut self) -> Scope<Self> {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["users", "set", self.name.as_str()]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        if let Some(role) = self.role {
            cmd.args(["--role", role]);
        }
        let assert = cmd
            .assert()
            .success()
            .stdout(contains!("Created service account '{self}'"));
        let output = assert.get_output();
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.api_key = Some(
            stdout
                .lines()
                .nth(1)
                .expect("Service account did not provide an API key")
                .to_string(),
        );
        Scope::new(self)
    }

    fn delete_cmd(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(cli_bin_path("cloudtruth"));
        cmd.args(["users", "delete", "--confirm", self.name.as_str()]);
        cmd
    }
}
