use crate::installation::InstallError;
use std::io;
use std::io::Write;
#[cfg(not(target_os = "windows"))]
#[rustfmt::skip]
use {
    crate::installation::binary_version,
    std::fs,
    std::process::Command,
    std::str,
    tempfile::tempdir,
};

#[cfg(target_os = "windows")]
pub fn install_latest_version(quiet: bool) -> Result<(), InstallError> {
    let text = include_str!("../../install.ps1");
    let result = powershell_script::run(text, false);
    match result {
        Ok(output) => {
            if !quiet {
                if let Some(stdout_str) = output.stdout() {
                    io::stdout().write_all(stdout_str.as_bytes())?;
                }
            }
            Ok(())
        }
        Err(err) => Err(InstallError::InstallFailed(err.to_string())),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn install_latest_version(quiet: bool) -> Result<(), InstallError> {
    let filename = format!("cloudtruth-cli-install-{}.sh", binary_version());
    let tempdir = tempdir()?;
    let fullpath = tempdir.path().join(filename);
    let fullname = fullpath.to_str().unwrap();
    let text = include_str!("../../install.sh");

    // write the install script to a file to a temporary directory
    fs::write(fullname, text)?;

    // attempt the chmod, and hope for success -- ignore failure
    let _ = Command::new("chmod").arg("a+x").arg(fullname).output();

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
