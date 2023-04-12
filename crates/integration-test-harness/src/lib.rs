#[macro_use]
extern crate derive_more;

pub mod assert;
mod backtrace;
pub mod command;
pub mod data;
pub mod error_handler;
#[cfg(feature = "macros")]
pub mod macros;
pub mod panic;
pub mod prelude;
mod source_span;
pub mod util;

/// Setup the integration test with error and panic handlers.
pub fn install() {
    error_handler::install_miette_error_handler();
    panic::set_panic_hook();
}
