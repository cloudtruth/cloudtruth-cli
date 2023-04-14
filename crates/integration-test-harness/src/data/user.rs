use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Scope, ScopedTestResourceExt, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct User<'d, 'r> {
    name: Name,
    description: Option<&'d str>,
    role: Option<&'r str>,
    api_key: Option<String>,
}

pub type ScopedUser<'d, 'r> = Scope<User<'d, 'r>>;

impl<'d, 'r> User<'d, 'r> {
    pub fn new(name: Name, description: Option<&'d str>, role: Option<&'r str>) -> Self {
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
    fn create(&mut self) {
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
        )
    }

    fn delete(&mut self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args(["users", "delete", "--confirm", self.name.as_str()])
            .assert()
            .success();
    }
}

impl<'d, 'r> From<&User<'d, 'r>> for String {
    /// Convert a User reference to a String by cloning its name.
    /// Needed for easy use of predicate functions.
    fn from(name: &User) -> Self {
        name.name().into()
    }
}

impl<'d, 'r> From<User<'d, 'r>> for String {
    /// Convert a User to a String representing its name
    fn from(user: User) -> Self {
        user.name.into()
    }
}

/// For more complex User data, use a UserBuilder to fill in the values and then call
/// build() or build_scoped() to create the User or Scope<User>, respectively.
pub struct UserBuilder<'d, 'r> {
    name: Name,
    description: Option<&'d str>,
    role: Option<&'r str>,
}

impl<'d, 'r> NameConstructors for UserBuilder<'d, 'r> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        UserBuilder::new(name)
    }
}

impl<'d, 'r> UserBuilder<'d, 'r> {
    /// Create a UserBuilder from a name.
    pub fn new<N: Into<Name>>(name: N) -> Self {
        UserBuilder {
            name: name.into(),
            description: None,
            role: None,
        }
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

    /// Build a Scope<User>. Equivalent to calling .build().scoped(). Because creating a Scope around a User creates the user on the server,
    /// this method does the same.
    pub fn build_scoped(self) -> Scope<User<'d, 'r>> {
        self.build().scoped()
    }

    /// Build a User. Does not automatically create the User on the server.
    pub fn build(self) -> User<'d, 'r> {
        User::new(self.name, self.description, self.role)
    }
}
