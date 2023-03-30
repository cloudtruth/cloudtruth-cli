mod harness;

use crate::harness::prelude::*;

#[test]
fn project_basic() {
    // miette::set_panic_hook();
    set_panic_hook();
    cloudtruth!("projects ls -v").assert().success();
}
