use std::process::{exit, Command};

use super::Cli;
use anyhow::*;

macro_rules! pytest_dir {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../pytest")
    };
}

impl Cli {
    pub fn cleanup(&self, args: &[String]) -> Result<()> {
        let mut py_cmd = Command::new("python3");
        py_cmd.current_dir(pytest_dir!());
        py_cmd.arg("cleanup.py");
        if self.verbose {
            py_cmd.arg("--verbose");
        }
        py_cmd.args(args);
        let status = py_cmd.spawn()?.wait()?;
        exit(status.code().unwrap_or_default())
    }
}
