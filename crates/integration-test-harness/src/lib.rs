#[macro_use]
extern crate derive_more;
pub mod assert;
mod backtrace;
pub mod command;
pub mod macros;
pub mod panic;
pub mod prelude;
pub mod scopes;
mod source_span;

pub fn install_harness() {
    panic::set_panic_hook();
}