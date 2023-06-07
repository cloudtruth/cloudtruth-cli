use std::{borrow::Cow, iter};

use crate::{
    config::{InstallType, ReleaseBuildConfig, ReleaseTestConfig, RunnerOs, TestOs},
    templates::DockerTemplate,
};
use itertools::{Either, Itertools};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseBuildMatrix<'c> {
    pub include: Vec<ReleaseBuildIncludes<'c>>,
}

#[derive(Serialize)]
pub struct ReleaseBuildIncludes<'c> {
    pub target: Cow<'c, str>,
    pub runner: RunnerOs,
}

impl<'c> FromIterator<&'c ReleaseBuildConfig<'c>> for ReleaseBuildMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseBuildConfig<'c>>>(value: T) -> Self {
        let include = value
            .into_iter()
            .map(
                |&ReleaseBuildConfig { ref target, runner }| ReleaseBuildIncludes {
                    target: Cow::from(target.as_ref()),
                    runner,
                },
            )
            .collect();
        ReleaseBuildMatrix { include }
    }
}

impl std::fmt::Display for ReleaseBuildMatrix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.include.iter().format(", "))
    }
}

impl std::fmt::Display for ReleaseBuildIncludes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.target)
    }
}

#[derive(Serialize)]
pub struct ReleaseTestMatrix<'c> {
    pub include: Vec<ReleaseTestIncludes<'c>>,
}
#[derive(Serialize)]
pub struct ReleaseTestIncludes<'c> {
    pub os: TestOs,
    pub runner: RunnerOs,
    pub version: Cow<'c, str>,
    pub platform: Option<Cow<'c, str>>,
    pub install_type: InstallType,
    pub dockerfile: Cow<'c, str>,
}

impl<'c> FromIterator<&'c ReleaseTestConfig<'c>> for ReleaseTestMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseTestConfig<'c>>>(value: T) -> Self {
        let include = value
            .into_iter()
            .flat_map(
                |&ReleaseTestConfig {
                     os,
                     ref versions,
                     install_type,
                     ref platforms,
                 }| {
                    let platforms = match platforms {
                        None => Either::Left(iter::once(None)),
                        Some(platforms) => Either::Right(platforms.iter().map(Option::Some)),
                    };
                    platforms.flat_map(move |platform| {
                        versions.iter().map(move |version| {
                            let platform = platform.map(|p| Cow::from(p.as_ref()));
                            let version = Cow::from(version.as_ref());
                            let dockerfile = Cow::Owned(
                                DockerTemplate {
                                    image: os,
                                    version: version.clone(),
                                    platform: platform.clone(),
                                }
                                .file_name(),
                            );
                            ReleaseTestIncludes {
                                os,
                                runner: RunnerOs::from(os),
                                version,
                                platform,
                                install_type,
                                dockerfile,
                            }
                        })
                    })
                },
            )
            .collect();
        ReleaseTestMatrix { include }
    }
}

impl std::fmt::Display for ReleaseTestMatrix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.include.iter().format(", "))
    }
}

impl std::fmt::Display for ReleaseTestIncludes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.os, self.version)
    }
}
