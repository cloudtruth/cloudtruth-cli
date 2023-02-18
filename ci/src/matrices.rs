use crate::config::{ReleaseBuildConfig, ReleaseTestConfig, RunnerOs, TestOs};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseBuildMatrix<'c> {
    pub target: Vec<&'c str>,
}

impl<'c> FromIterator<&'c ReleaseBuildConfig<'c>> for ReleaseBuildMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseBuildConfig<'c>>>(value: T) -> Self {
        Self {
            target: value.into_iter().map(|i| i.target.as_ref()).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct ReleaseTestMatrix<'c> {
    pub os: Vec<TestOs>,
    pub includes: Vec<ReleaseTestIncludes<'c>>,
}

#[derive(Serialize)]
pub struct ReleaseTestIncludes<'c> {
    pub os: TestOs,
    pub runner: RunnerOs,
    pub version: &'c str,
}

impl<'c> FromIterator<&'c ReleaseTestConfig<'c>> for ReleaseTestMatrix<'c> {
    fn from_iter<T: IntoIterator<Item = &'c ReleaseTestConfig<'c>>>(value: T) -> Self {
        let mut matrix = ReleaseTestMatrix {
            os: Vec::new(),
            includes: Vec::new(),
        };
        for test in value {
            let &ReleaseTestConfig {
                os, ref versions, ..
            } = test;
            matrix.os.push(os);
            matrix
                .includes
                .extend(versions.iter().map(|version| ReleaseTestIncludes {
                    os,
                    runner: RunnerOs::from(os),
                    version,
                }));
        }
        matrix
    }
}
