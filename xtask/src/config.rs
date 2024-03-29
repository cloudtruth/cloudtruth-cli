use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

/// config.yaml file
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config<'c> {
    #[serde(borrow)]
    pub release_builds: Vec<ReleaseBuildConfig<'c>>,
    pub release_tests: Vec<ReleaseTestConfig<'c>>,
}

/// release-builds section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseBuildConfig<'c> {
    pub runner: RunnerOs,
    #[serde(borrow)]
    pub target: Cow<'c, str>,
}

/// release-tests section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseTestConfig<'c> {
    pub os: TestOs,
    #[serde(rename = "type")]
    pub install_type: InstallType,
    #[serde(borrow)]
    pub versions: Vec<Cow<'c, str>>,
    pub platforms: Option<Vec<Cow<'c, str>>>,
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

derive_display_from_serialize!(RunnerOs);

impl From<TestOs> for RunnerOs {
    // Choose the corresponding GitHub runner for a Test OS
    fn from(test_os: TestOs) -> Self {
        use RunnerOs::*;
        use TestOs::*;
        match test_os {
            Macos => MacosLatest,
            Windows => WindowsLatest,
            _ => UbuntuLatest,
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TestOs {
    Alpine,
    #[serde(rename = "arm32v6/alpine")]
    AlpineArm32v6,
    #[serde(rename = "arm32v7/alpine")]
    AlpineArm32v7,
    #[serde(rename = "arm64v8/alpine")]
    AlpineArm64v8,
    RockyLinux,
    Centos,
    Debian,
    Ubuntu,
    Macos,
    Windows,
}

derive_display_from_serialize!(TestOs);

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

derive_display_from_serialize!(InstallType);
