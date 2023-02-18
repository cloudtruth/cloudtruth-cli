use crate::config::{ReleaseTest, TestOs};
use askama::Template;

/// Template for generating the installation test Dockerfiles
#[derive(Template)]
#[template(path = "Dockerfile", escape = "none")]
//#[template(print = "code")] //uncomment for debugging generated code
pub struct DockerTemplate<'c> {
    pub os: TestOs,
    pub version: &'c str,
}

impl<'c> DockerTemplate<'c> {
    pub fn from_config(
        release_test: &'c ReleaseTest<'c>,
    ) -> impl Iterator<Item = DockerTemplate<'c>> {
        let &ReleaseTest {
            os, ref versions, ..
        } = release_test;
        versions
            .into_iter()
            .map(move |&version| DockerTemplate { os, version })
    }
}
