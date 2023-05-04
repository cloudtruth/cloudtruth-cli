pub mod cli;
pub mod install;
pub mod install_errors;
pub mod package_manager;
pub mod version;

pub use cli::Cli;
pub use install::install_latest_version;
pub use install_errors::InstallError;
pub use package_manager::find_package_managers;
pub use version::{binary_version, get_latest_version};

macro_rules! verbose {
    ($($expr:tt)*) => { if $crate::cli::verbose() { println!($($expr)*)}}
}
pub(crate) use verbose;
