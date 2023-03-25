mod install;
mod install_errors;
mod version;

pub use install::install_latest_version;
pub use install_errors::InstallError;
pub use version::{binary_version, get_latest_version};
