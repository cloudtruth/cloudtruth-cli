use cloudtruth_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_generate_password_basic() {
    // default length
    cloudtruth!("generate pw")
        .assert()
        .success()
        .stdout(len(16));

    for length in [8, 15, 75, 4095] {
        cloudtruth!("generate pw --length {length}")
            .assert()
            .success()
            .stdout(len(length + 1));
    }

    let flag_tests = [
        ("lowercase", "require_lowercase"),
        ("uppercase", "require_uppercase"),
        ("number", "require_numbers"),
        ("symbol", "require_symbols"),
        ("space", "require_spaces"),
        ("hardware", "require_hardware_generation"),
    ];
    for (flag, query) in flag_tests {
        cloudtruth!("generate pw")
            .rest_debug()
            .assert()
            .success()
            .stdout(not(contains!("{query}")));
        cloudtruth!("generate pw --{flag}")
            .rest_debug()
            .assert()
            .success()
            .stdout(contains!("{query}=true"));
        cloudtruth!("generate pw --no-{flag}")
            .rest_debug()
            .assert()
            .success()
            .stdout(contains!("{query}=false"));
    }
    // test all flags together
    let all_positive_flags = flag_tests.map(|(flag, _)| format!("--{flag}")).join(" ");
    let all_negative_flags = flag_tests.map(|(flag, _)| format!("--no-{flag}")).join(" ");
    cloudtruth!("generate pw {all_positive_flags} {all_negative_flags}")
        .rest_debug()
        .assert()
        .success()
        .stdout(contains_all(
            flag_tests.map(|(_, query)| format!("{query}=true")),
        ));

    // negative cases
    cloudtruth!("generate pw --length 7")
        .assert()
        .failure()
        .stderr(contains("Password must be 8 or more characters"));

    cloudtruth!("generate pw --length 4096")
        .assert()
        .failure()
        .stderr(contains("Password must be less than 4096 character"));
}
