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
        let (target, include) = value
            .into_iter()
            .map(|&ReleaseBuildConfig { ref target, runner }| {
                (
                    target.as_ref(),
                    ReleaseBuildIncludes {
                        target: target.as_ref(),
                        runner,
                    },
                )
            })
            .unzip();
        Self { target, include }
    }
}

impl std::fmt::Display for ReleaseBuildMatrix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.target.join(" "))
    }
}

#[derive(Serialize)]
pub struct ReleaseTestMatrix<'c> {
    pub os: Vec<TestOs>,
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
            os: Vec::new(),
            include: Vec::new(),
        };
        for test in value {
            let &ReleaseTestConfig {
                os,
                ref versions,
                install_type,
                ..
            } = test;
            matrix.os.push(os);
            matrix
                .include
                .extend(versions.iter().map(|version| ReleaseTestIncludes {
                    os,
                    runner: RunnerOs::from(os),
                    version,
                    install_type,
                }));
        }
        matrix
    }
}
