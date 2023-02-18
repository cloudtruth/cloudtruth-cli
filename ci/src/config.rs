use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::matrix_generator::HasSortKey;

/// config.yaml file
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config<'c> {
    #[serde(borrow)]
    pub release_builds: Vec<ReleaseBuild<'c>>,
    pub release_tests: Vec<ReleaseTest<'c>>,
}

/// release-builds section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseBuild<'c> {
    pub runner: RunnerOs,
    pub target: &'c str,
}

impl<'c> HasSortKey for ReleaseBuild<'c> {
    type Key = RunnerOs;
    fn sort_key(&self) -> Self::Key {
        self.runner
    }
}

/// release-tests section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseTest<'c> {
    pub os: TestOs,
    #[serde(rename = "type")]
    pub install_type: InstallType,
    #[serde(borrow)]
    pub versions: Vec<&'c str>,
}

impl<'c> HasSortKey for ReleaseTest<'c> {
    type Key = InstallType;
    fn sort_key(&self) -> Self::Key {
        self.install_type
    }
}

/// GitHub Actions runners
#[allow(clippy::enum_variant_names)]
#[derive(Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RunnerOs {
    UbuntuLatest,
    MacosLatest,
    WindowsLatest,
}

impl From<TestOs> for RunnerOs {
    // Choose the corresponding GitHub runner for a Test OS
    fn from(test_os: TestOs) -> Self {
        use RunnerOs::*;
        use TestOs::*;
        match test_os {
            Alpine | RockyLinux | Debian | Ubuntu => UbuntuLatest,
            Macos => MacosLatest,
            Windows => WindowsLatest,
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TestOs {
    Alpine,
    RockyLinux,
    Debian,
    Ubuntu,
    Macos,
    Windows,
}

/// Used for template rendering and output of Dockerfile names
impl Display for TestOs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TestOs::*;
        match self {
            // linux OS strings are primarily used for generating Dockerfile names
            Alpine => write!(f, "alpine"),
            RockyLinux => write!(f, "rockylinux"),
            Debian => write!(f, "debian"),
            Ubuntu => write!(f, "ubuntu"),
            // the uppercase format for macOS and Windows matches GH Actions OS strings
            Macos => write!(f, "macOS"),
            Windows => write!(f, "Windows"),
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InstallType {
    /// Install with install.sh in Docker container
    Docker,
    /// Install with install.sh
    Shell,
    /// Install with install.ps1
    PowerShell,
}
