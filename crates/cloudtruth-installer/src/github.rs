use bytes::Bytes;
use color_eyre::{eyre::eyre, Result};
use futures_core::Stream;
use futures_util::StreamExt;
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

/// Get tag name of latest GitHub release
pub async fn get_latest_version() -> Result<String> {
    Ok(octocrab::instance()
        .repos("cloudtruth", "cloudtruth-cli")
        .releases()
        .get_latest()
        .await?
        .tag_name)
}

/// Download asset from GitHub
async fn get_release_asset(
    version: &str,
    asset_name: &str,
) -> Result<impl Stream<Item = reqwest::Result<Bytes>>> {
    let github = octocrab::instance();
    let download_url = github
        .repos("cloudtruth", "cloudtruth-cli")
        .releases()
        .get_by_tag(version)
        .await?
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .map(|asset| asset.browser_download_url)
        .ok_or_else(|| eyre!("Could not find release asset {asset_name} in release {version}"))?;
    Ok(reqwest::get(download_url).await?.bytes_stream())
}

pub async fn download_release_asset(
    version: &str,
    asset_name: &str,
    download_path: &Path,
) -> Result<()> {
    let mut f = File::create(download_path).await?;
    let mut stream = get_release_asset(version, asset_name).await?;
    while let Some(chunk) = stream.next().await {
        f.write_all(&chunk?).await?;
    }
    Ok(())
}

// Get package name for the current build target, version, and file extension
pub fn asset_name(version: &str, ext: &str) -> String {
    const TARGET: &str = env!("TARGET");
    format!("cloudtruth-{version}-{TARGET}.{ext}")
}
