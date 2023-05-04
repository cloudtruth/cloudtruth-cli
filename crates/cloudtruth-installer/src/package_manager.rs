use color_eyre::Result;
use core::fmt;

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use which::which;

use crate::verbose;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum PackageManager {
    Apt,
    Apk,
    Yum,
    Zypper,
    Brew,
}

impl PackageManager {
    #[cfg(not(target_os = "macos"))]
    pub const LIST: &[PackageManager] = &[
        PackageManager::Apt,
        PackageManager::Apk,
        PackageManager::Yum,
        PackageManager::Zypper,
        PackageManager::Brew,
    ];
    #[cfg(target_os = "macos")]
    pub const LIST: &[PackageManager] = &[
        PackageManager::Brew,
        PackageManager::Apt,
        PackageManager::Apk,
        PackageManager::Yum,
        PackageManager::Zypper,
    ];

    /// Iterator over list of possible package managers.
    /// Ordering is platform-specific based on which package managers
    /// are commonly used on those systems.
    pub fn iter() -> impl Iterator<Item = Self> {
        Self::LIST.iter().copied()
    }

    pub fn cmd_name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Apk => "apk",
            PackageManager::Yum => "yum",
            PackageManager::Zypper => "zypper",
            PackageManager::Brew => "brew",
        }
    }

    pub fn package_ext(&self) -> &'static str {
        match self {
            PackageManager::Apt => "deb",
            PackageManager::Apk => "apk",
            PackageManager::Yum => "rpm",
            PackageManager::Zypper => "rpm",
            PackageManager::Brew => "bottle.tar.gz",
        }
    }

    pub fn find_bin_path(&self) -> Option<PathBuf> {
        which(self.cmd_name()).ok()
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cmd_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
#[display(fmt = "{} ({:?})", package_manager, bin_path)]
pub struct PackageManagerBin {
    package_manager: PackageManager,
    bin_path: PathBuf,
}

impl PackageManagerBin {
    pub fn find(package_manager: PackageManager) -> Option<PackageManagerBin> {
        package_manager
            .find_bin_path()
            .map(|bin_path| PackageManagerBin {
                package_manager,
                bin_path,
            })
    }

    pub fn package_manager(&self) -> PackageManager {
        self.package_manager
    }

    pub fn cmd_name(&self) -> &'static str {
        self.package_manager.cmd_name()
    }

    pub fn package_ext(&self) -> &'static str {
        self.package_manager.package_ext()
    }

    pub fn bin_path(&self) -> &Path {
        &self.bin_path
    }

    pub fn install(&self, package: &Path) -> Result<()> {
        match self.package_manager {
            PackageManager::Apt => apt::install(self, package),
            PackageManager::Apk => apk::install(self, package),
            PackageManager::Yum => yum::install(self, package),
            PackageManager::Zypper => zypper::install(self, package),
            PackageManager::Brew => brew::install(self, package),
        }
    }

    pub fn check_status(&self) -> bool {
        match self.package_manager {
            PackageManager::Apt => apt::status(self),
            PackageManager::Apk => apk::status(self),
            PackageManager::Yum => yum::status(self),
            PackageManager::Zypper => zypper::status(self),
            PackageManager::Brew => brew::status(self),
        }
    }
}

pub fn find_package_managers() -> impl Iterator<Item = PackageManagerBin> {
    PackageManager::iter()
        .inspect(|pm| verbose!("Searching for {pm}"))
        .filter_map(PackageManagerBin::find)
        .inspect(|pm| verbose!("Found {pm}"))
        .filter(PackageManagerBin::check_status)
}

mod apt {
    use color_eyre::Result;
    use std::{path::Path, process::Command};

    use super::PackageManagerBin;

    pub fn install(_pm: &PackageManagerBin, _package: &Path) -> Result<()> {
        todo!()
    }

    pub fn status(pm: &PackageManagerBin) -> bool {
        super::check_status(pm.cmd_name(), &mut Command::new(pm.bin_path()))
    }
}

mod apk {
    use color_eyre::Result;
    use std::{path::Path, process::Command};

    use super::PackageManagerBin;

    pub fn install(_pm: &PackageManagerBin, _package: &Path) -> Result<()> {
        todo!()
    }

    pub fn status(pm: &PackageManagerBin) -> bool {
        super::check_status(pm.cmd_name(), &mut Command::new(pm.bin_path()))
    }
}

mod yum {
    use color_eyre::Result;
    use std::{path::Path, process::Command};

    use super::{check_status, PackageManagerBin};

    pub fn install(_pm: &PackageManagerBin, _package: &Path) -> Result<()> {
        todo!()
    }

    pub fn status(pm: &PackageManagerBin) -> bool {
        check_status(pm.cmd_name(), &mut Command::new(pm.bin_path()))
    }
}

mod zypper {
    use color_eyre::Result;
    use std::{path::Path, process::Command};

    use super::{check_status, PackageManagerBin};

    pub fn install(_pm: &PackageManagerBin, _package: &Path) -> Result<()> {
        todo!()
    }

    pub fn status(pm: &PackageManagerBin) -> bool {
        check_status(pm.cmd_name(), &mut Command::new(pm.bin_path()))
    }
}

mod brew {
    use color_eyre::Result;
    use std::{path::Path, process::Command};

    use super::{check_status, PackageManagerBin};

    pub fn install(_pm: &PackageManagerBin, _package: &Path) -> Result<()> {
        todo!()
    }

    pub fn status(pm: &PackageManagerBin) -> bool {
        check_status(pm.cmd_name(), Command::new(pm.bin_path()).arg("commands"))
    }
}

fn check_status(cmd_name: &str, cmd: &mut Command) -> bool {
    verbose!("Checking status of {cmd_name}");
    match cmd.stderr(Stdio::null()).stdout(Stdio::null()).status() {
        Ok(status) => {
            verbose!("{cmd_name} {status}");
            status.success()
        }
        Err(err) => {
            verbose!("{cmd_name} ERROR: {err}");
            false
        }
    }
}
