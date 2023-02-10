use std::path::Path;

#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new("target/debug/cloudtruth"))
        .case("tests/commands/*.md");
}
