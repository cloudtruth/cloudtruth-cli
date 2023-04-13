use std::path::Path;

use crate::{
    command::{cli_bin_path, Command},
    contains,
    data::{Name, NameConstructors, Project, Scope, ScopedTestResourceExt, TestResource},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", name)]
pub struct Template<'p, 'd, 'b> {
    name: Name,
    project: &'p Name,
    description: Option<&'d str>,
    body: &'b Path,
}

impl<'p, 'd, 'b> Template<'p, 'd, 'b> {
    pub fn new<P: AsRef<Path> + ?Sized>(
        name: Name,
        project: &'p Name,
        description: Option<&'d str>,
        body: &'b P,
    ) -> Self {
        Self {
            name,
            project,
            description,
            body: body.as_ref(),
        }
    }

    pub fn rename<N: Into<Name>>(&mut self, name: N) -> &mut Self {
        let name = name.into();
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "--project",
                self.project.as_str(),
                "templates",
                "set",
                self.name.as_str(),
                "--rename",
                name.as_str(),
            ])
            .assert()
            .success()
            .stdout(contains!("Updated template '{name}'"));
        self.name = name;
        self
    }
}

impl<'p, 'd, 'b> TestResource for Template<'p, 'd, 'b> {
    fn name(&self) -> &Name {
        &self.name
    }

    fn create(&self) {
        let mut cmd = Command::new(cli_bin_path("cloudtruth"));
        cmd.args([
            "--project",
            self.project.as_str(),
            "templates",
            "set",
            self.name.as_str(),
        ]);
        if let Some(desc) = self.description {
            cmd.args(["--desc", desc]);
        }
        cmd.arg("--body").arg(self.body);
        cmd.assert().success();
    }

    fn delete(&self) {
        Command::new(cli_bin_path("cloudtruth"))
            .args([
                "--project",
                self.project.as_str(),
                "templates",
                "delete",
                "--confirm",
                self.name.as_str(),
            ])
            .assert()
            .success();
    }
}

pub struct TemplateBuilder<'p, 'd, 'b> {
    name: Name,
    project: Option<&'p Name>,
    description: Option<&'d str>,
    body: Option<&'b Path>,
}

impl<'p, 'd, 'b> NameConstructors for TemplateBuilder<'p, 'd, 'b> {
    fn from_name<N: Into<Name>>(name: N) -> Self {
        Self::new(name)
    }
}

impl<'p, 'd, 'b> TemplateBuilder<'p, 'd, 'b> {
    pub fn new<N: Into<Name>>(name: N) -> Self {
        Self {
            name: name.into(),
            project: None,
            description: None,
            body: None,
        }
    }

    /// Set the templates project.
    pub fn project(mut self, project: &'p Project) -> Self {
        self.project = Some(project.name());
        self
    }

    /// Set the templates description.
    pub fn description(mut self, description: &'d str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the templates body from a file path.
    pub fn body<P: AsRef<Path> + ?Sized>(mut self, body: &'b P) -> Self {
        self.body = Some(body.as_ref());
        self
    }

    /// Build a Scope<Template>. Equivalent to calling .build().scoped(). Because creating a Scope around a Template creates the template on the server,
    /// this method does the same.
    pub fn build_scoped(self) -> Scope<Template<'p, 'd, 'b>> {
        self.build().scoped()
    }

    /// Build a Template. Does not automatically create the Template on the server.
    pub fn build(self) -> Template<'p, 'd, 'b> {
        Template::new(
            self.name,
            self.project
                .expect("Couldn't build Template: no project provided"),
            self.description,
            self.body
                .expect("Couldn't build Template: no body provided"),
        )
    }
}
