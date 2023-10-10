use std::process::Command;

macro_rules! pytest_dir {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/pytest")
    };
}

#[test]
fn test_integrations_pytest() -> std::io::Result<()> {
    let mut handle = Command::new("python3")
        .current_dir(pytest_dir!())
        .args(["live_test.py", "--file", "test_integrations.py"])
        .spawn()?;
    let status = handle.wait()?;
    assert!(status.success());
    Ok(())
}
