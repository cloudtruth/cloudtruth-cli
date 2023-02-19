use std::io::Write;

use crate::config::{ReleaseTestConfig, TestOs};
use anyhow::*;
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
    pub fn file_name(&self) -> String {
        format!("Dockerfile.{}-{}", self.os, self.version)
    }

    pub fn from_release_test_config(
        release_test: &'c ReleaseTestConfig<'c>,
    ) -> impl Iterator<Item = DockerTemplate<'c>> {
        let &ReleaseTestConfig {
            os, ref versions, ..
        } = release_test;
        versions
            .iter()
            .map(move |version| DockerTemplate { os, version })
    }

    pub fn write_dockerfile<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(self.render()?.as_bytes())?;
        Ok(())
    }
}
