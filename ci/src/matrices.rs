use crate::config::{RunnerOs, TestOs};
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseBuildMatrix<'conf> {
    pub target: Vec<&'conf str>,
}

#[derive(Serialize)]
pub struct ReleaseTestMatrix<'conf> {
    pub os: Vec<TestOs>,
    pub includes: Vec<ReleaseTestIncludes<'conf>>,
}

#[derive(Serialize)]
pub struct ReleaseTestIncludes<'conf> {
    pub os: TestOs,
    pub runner: RunnerOs,
    pub version: &'conf str,
}
