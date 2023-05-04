use clap::Parser;
use cloudtruth_installer::{
    cli, find_package_managers, init_globals, package_manager::PackageManagerBin, Cli,
};

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    init_globals(&cli);
    let pkg_manager = if cli::non_interactive() {
        // When running non-interactive, take first available package manager
        find_package_managers().next().unwrap()
    } else {
        let mut pkg_managers: Vec<PackageManagerBin> = find_package_managers().collect();
        if pkg_managers.len() == 1 {
            pkg_managers.swap_remove(0)
        } else {
            // Prompt user for package manager choice
            todo!()
        }
    };
    println!("{}", pkg_manager);
    Ok(())
}
