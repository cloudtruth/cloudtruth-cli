use std::io::Write;

use crate::config::{Config, InstallType, ReleaseTestConfig, TestOs};
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

    // Generate sequence of Dockerfiles from the release-tests config
    pub fn iter_from_config(config: &'c Config<'c>) -> impl Iterator<Item = DockerTemplate<'c>> {
        config
            .release_tests
            .iter()
            .filter(|t| t.install_type == InstallType::Docker)
            .flat_map(DockerTemplate::from_release_test_config)
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
