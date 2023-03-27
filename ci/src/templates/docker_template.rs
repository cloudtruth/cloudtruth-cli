use std::io::Write;

use crate::config::{InstallType, ReleaseTestConfig, TestOs};
use anyhow::*;
use askama::Template;
use tokio::task::spawn_blocking;

/// Template for generating the installation test Dockerfiles
#[derive(Debug, Template)]
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
    pub fn iter_from_config(
        config: &'c [ReleaseTestConfig<'c>],
    ) -> impl Iterator<Item = DockerTemplate<'c>> {
        config
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

    pub async fn write_dockerfile_async<W: Write + Send + 'static>(
        &self,
        mut writer: W,
    ) -> Result<()> {
        let str = self.render()?;
        spawn_blocking(move || writer.write_all(str.as_bytes()).map_err(anyhow::Error::new)).await?
    }
}
