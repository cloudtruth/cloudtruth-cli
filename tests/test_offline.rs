use std::path::Path;
use integration_test_harness::prelude::*;

#[test]
fn help_text() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(cli_bin_path!()))
        .case("examples/help-text/*.md");
}
