use crate::{cli::InstallCommand, github, package_manager::choose_package_manager};

use color_eyre::Result;
use flate2::read::GzDecoder;
use std::{fs::File, path::Path};
use tar::Archive;
use tempfile::tempdir;

pub fn unpack_tar_gz(src_file_path: &Path, dest_file_path: &Path) -> Result<()> {
    let tar_gz = File::open(src_file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dest_file_path)?;
    Ok(())
}

pub async fn install(cmd: InstallCommand) -> Result<()> {
    let pkg_manager = choose_package_manager();
    let version = match cmd.version {
        Some(version) => version,
        None => github::get_latest_version().await?,
    };
    let tmp_dir = tempdir()?;
    let ext = if let Some(pkg_manager) = &pkg_manager {
        pkg_manager.package_ext()
    } else if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let asset_name = github::asset_name(&version, ext);
    let download_path = tmp_dir.path().join(&asset_name);
    github::download_release_asset(&version, &asset_name, &download_path).await?;
    if let Some(pkg_manager) = pkg_manager {
        pkg_manager.install(&download_path)?
    } else if cfg!(target_os = "windows") {
        todo!()
    } else {
        unpack_tar_gz(&download_path, tmp_dir.path())?;
        println!("{:?}", tmp_dir.path());
    };
    Ok(())
}
