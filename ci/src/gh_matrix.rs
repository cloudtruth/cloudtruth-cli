use crate::config::{InstallType, ReleaseBuildConfig, ReleaseTestConfig, RunnerOs, TestOs};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseBuildMatrix<'c> {
    pub target: Vec<&'c str>,
    pub include: Vec<ReleaseBuildIncludes<'c>>,
}

#[derive(Serialize)]
pub struct ReleaseBuildIncludes<'c> {
    pub target: &'c str,
    pub runner: RunnerOs,
}

impl<'c> FromIterator<&'c ReleaseBuildConfig<'c>> for ReleaseBuildMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseBuildConfig<'c>>>(value: T) -> Self {
        let mut matrix = ReleaseBuildMatrix {
            target: Vec::new(),
            include: Vec::new(),
        };
        for &ReleaseBuildConfig { ref target, runner } in value {
            matrix.target.push(target);
            matrix.include.push(ReleaseBuildIncludes { target, runner });
        }
        matrix
    }
}

impl std::fmt::Display for ReleaseBuildMatrix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.target.join(" "))
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

impl std::fmt::Display for ReleaseTestMatrix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in &self.include {
            write!(f, "{}-{} ", i.os, i.version)?;
        }
        std::fmt::Result::Ok(())
    }
}

impl<'c> FromIterator<&'c ReleaseTestConfig<'c>> for ReleaseTestMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseTestConfig<'c>>>(value: T) -> Self {
        let mut matrix = ReleaseTestMatrix {
            include: Vec::new(),
        };
        for &ReleaseTestConfig {
            os,
            ref versions,
            install_type,
        } in value
        {
            for version in versions {
                matrix.include.push(ReleaseTestIncludes {
                    os,
                    runner: RunnerOs::from(os),
                    version,
                    install_type,
                });
            }
        }
        matrix
    }
}
