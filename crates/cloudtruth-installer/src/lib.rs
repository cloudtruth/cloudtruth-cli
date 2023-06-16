mod cli;
mod github;
mod install;
mod package_manager;

use cli::Subcommand;
use color_eyre::Result;
use install::install;

macro_rules! verbose {
    ($($expr:tt)*) => { if $crate::cli::verbose() { println!($($expr)*)}}
}
pub(crate) use verbose;

pub async fn cloudtruth_installer_cli() -> Result<()> {
    let cli = cli::parse();
    #[allow(irrefutable_let_patterns)]
    if let Subcommand::Install(install_cmd) = cli.command {
        install(install_cmd).await?;
    }
    Ok(())
}
