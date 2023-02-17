use serde::{Deserialize, Serialize};

/// config.yaml file
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub release_builds: Vec<ReleaseBuild>,
    pub release_tests: Vec<ReleaseTest>,
}

/// release-builds section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseBuild {
    pub runner: RunnerOs,
    pub target: String,
}

/// release-tests section of config.yaml
#[derive(Deserialize, Debug)]
pub struct ReleaseTest {
    pub os: TestOs,
    #[serde(rename = "type")]
    pub install_type: InstallType,
    pub versions: Vec<String>,
}
/// GitHub Actions runners
#[derive(Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RunnerOs {
    UbuntuLatest,
    MacosLatest,
    WindowsLatest,
}

impl RunnerOs {
    // Choose the corresponding GitHub runner for a Test OS
    pub fn from_test_os(test_os: TestOs) -> RunnerOs {
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
