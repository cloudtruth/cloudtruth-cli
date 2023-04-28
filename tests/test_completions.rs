use integration_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_completions() {
    for shell in ["zsh", "bash", "fish", "powershell", "elvish"] {
        cloudtruth!("completions {shell}").assert().success();
    }
}
