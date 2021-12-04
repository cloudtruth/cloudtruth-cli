use crate::installation::{binary_version, InstallError};
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str;
use tempfile::tempdir;

#[cfg(target_os = "windows")]
const INSTALL_TEXT: &[u8] = include_bytes!(".,/../install.ps1");

#[cfg(not(target_os = "windows"))]
const INSTALL_TEXT: &[u8] = include_bytes!("../../install.sh");

pub fn install_latest_version(quiet: bool) -> Result<(), InstallError> {
    let filename = format!("cloudtruth-cli-install-{}", binary_version());
    let tempdir = tempdir()?;
    let fullpath = tempdir.path().join(filename);
    let fullname = fullpath.to_str().unwrap();

    // write the install script to a file to a temporary directory
    fs::write(fullname, INSTALL_TEXT)?;
    fs::set_permissions(fullname, std::fs::Permissions::from_mode(0o755))?;

    // now, actually run the installation script
    let result = Command::new(fullname).output();
    match result {
        Ok(output) => match output.status.success() {
            true => {
                if !quiet {
                    io::stdout().write_all(&output.stdout)?;
                }
                Ok(())
            }
            false => {
                if !quiet {
                    io::stdout().write_all(&output.stdout)?;
                }
                let stderr = str::from_utf8(&output.stderr)?;
                Err(InstallError::InstallFailed(stderr.to_string()))
            }
        },
        Err(err) => Err(InstallError::FailedToRunInstall(err.to_string())),
    }
}
