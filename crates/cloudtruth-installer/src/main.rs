use cloudtruth_installer::cloudtruth_installer_cli;

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cloudtruth_installer_cli().await?;
    Ok(())
}
