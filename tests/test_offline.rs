use std::path::Path;

#[cfg(target_os = "windows")]
const BIN_PATH: &str = "target/debug/cloudtruth.exe";
#[cfg(not(target_os = "windows"))]
const BIN_PATH: &str = "target/debug/cloudtruth";

#[test]
fn help_text() {
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(BIN_PATH))
        .case("examples/help-text/*.md");
}
