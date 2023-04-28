use integration_test_harness::prelude::*;
use std::str;

#[test]
#[use_harness]
fn test_versions() {
    let version_cmd = cloudtruth!("--version").assert().success();
    let version = str::from_utf8(&version_cmd.get_output().stdout)
        .unwrap()
        .split(' ')
        .last()
        .unwrap()
        .trim();

    cloudtruth!("version check")
        .assert()
        .success()
        .stdout(contains(version));

    cloudtruth!("version install -f").assert();

    cloudtruth!("ve get")
        .assert()
        .success()
        .stdout(contains!("Current CLI version {version}"));

    cloudtruth!("v get --latest")
        .assert()
        .success()
        .stdout(contains("Latest CLI version"));
}
