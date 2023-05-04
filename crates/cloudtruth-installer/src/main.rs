use cloudtruth_installer::{cli, find_package_managers, package_manager::PackageManagerBin};

pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cli::parse();
    let pkg_manager = if !cli::interactive() {
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
