use std::{borrow::Cow, io::Write, iter};

use crate::config::{InstallType, ReleaseTestConfig, TestOs};
use anyhow::*;
use askama::Template;
use itertools::Either;
use tokio::task::spawn_blocking;

/// Template for generating the installation test Dockerfiles
#[derive(Debug, Template)]
#[template(path = "Dockerfile", escape = "none")]
//#[template(print = "code")] //uncomment for debugging generated code
pub struct DockerTemplate<'c> {
    pub image: TestOs,
    pub version: Cow<'c, str>,
    pub platform: Option<Cow<'c, str>>,
}

impl<'c> DockerTemplate<'c> {
    pub fn file_name(&self) -> String {
        let image = // sanitize slashes in image name
        self.image
                .to_string()
                .replace(|c| c == '/' || c == '\\', "-");
        let version = self.version.as_ref();
        let platform_suffix = self
            .platform
            .as_ref()
            .map(|p| format!("-{p}"))
            .unwrap_or_default();
        format!("Dockerfile.{image}-{version}{platform_suffix}")
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
            os,
            ref versions,
            ref platforms,
            ..
        } = release_test;
        let platforms = match platforms {
            None => Either::Left(iter::once(None)),
            Some(platforms) => Either::Right(platforms.iter().map(Option::Some)),
        };
        platforms.flat_map(move |platform| {
            versions.iter().map(move |version| DockerTemplate {
                image: os,
                version: Cow::from(version.as_ref()),
                platform: platform.map(|p| Cow::from(p.as_ref())),
            })
        })
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
