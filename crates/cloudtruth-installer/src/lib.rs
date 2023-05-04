mod cli;
mod install;
mod install_errors;
mod package_manager;
mod version;

pub use cli::{init_globals, Cli};
pub use install::install_latest_version;
pub use install_errors::InstallError;
pub use package_manager::find_package_managers;
pub use version::{binary_version, get_latest_version};

macro_rules! verbose {
    ($($expr:tt)*) => { if $crate::cli::verbose() { println!($($expr)*)}}
}
pub(crate) use verbose;
