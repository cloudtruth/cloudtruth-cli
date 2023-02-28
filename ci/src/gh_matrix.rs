use crate::config::{InstallType, ReleaseBuildConfig, ReleaseTestConfig, RunnerOs, TestOs};
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseBuildMatrix<'c> {
    pub include: Vec<ReleaseBuildIncludes<'c>>,
}

#[derive(Serialize)]
pub struct ReleaseBuildIncludes<'c> {
    pub target: &'c str,
    pub runner: RunnerOs,
}

impl<'c> FromIterator<&'c ReleaseBuildConfig<'c>> for ReleaseBuildMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseBuildConfig<'c>>>(value: T) -> Self {
        let include = value
            .into_iter()
            .map(
                |&ReleaseBuildConfig { ref target, runner }| ReleaseBuildIncludes {
                    target: target.as_ref(),
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
    pub version: &'c str,
    pub install_type: InstallType,
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
                 }| {
                    versions.iter().map(move |version| ReleaseTestIncludes {
                        os,
                        runner: RunnerOs::from(os),
                        version,
                        install_type,
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
