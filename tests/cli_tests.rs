use std::path::Path;

#[cfg(target_os = "windows")]
const BIN_PATH: &str = "target/debug/cloudtruth.exe";
#[cfg(not(target_os = "windows"))]
const BIN_PATH: &str = "target/debug/cloudtruth";

#[test]
fn basic_tests() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(BIN_PATH))
        .case("tests/basic/*.md");
}

#[test]
fn help_tests() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(BIN_PATH))
        .case("tests/help/*.md");
}
