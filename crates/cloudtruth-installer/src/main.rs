use clap::Parser;
use cloudtruth_installer::{find_package_managers, init_globals, Cli};

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    init_globals(&cli);
    println!("{cli:?}");
    let pkg_managers = find_package_managers();
    println!("Found package managers:");
    for pm in pkg_managers {
        println!("{pm}");
    }
    Ok(())
}
