use std::{env, error::Error, path::PathBuf, process::Command};

#[test]
fn test_integrations_pytest() -> Result<(), Box<dyn Error>> {
    let live_test = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("pytest/live_test.py");
    let mut handle = Command::new("python3")
        .args([
            live_test.to_string_lossy().as_ref(),
            "--file",
            "test_integrations.py",
        ])
        .spawn()?;
    let status = handle.wait()?;
    assert!(status.success());
    Ok(())
}
