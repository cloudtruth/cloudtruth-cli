use std::{env, error::Error, path::PathBuf, process::Command};

#[test]
fn test_integrations_pytest() -> Result<(), Box<dyn Error>> {
    let cargo_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let live_test = cargo_manifest_dir.join("pytest/live_test.py");
    let mut handle = Command::new("python3")
        .current_dir(cargo_manifest_dir)
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
