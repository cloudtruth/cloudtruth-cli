use integration_test_harness::prelude::*;

#[integration_test]
fn project_basic() {
    cloudtruth!("projects ls -v").assert().success();
}
