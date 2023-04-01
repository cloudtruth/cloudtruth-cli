use integration_test_harness::prelude::*;
use std::path::Path;

#[test]
fn help_text() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(cli_bin_path!()))
        .case("examples/help-text/*.md");
}
