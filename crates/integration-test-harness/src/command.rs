use cloudtruth_config::{CT_API_KEY, CT_REST_DEBUG, CT_REST_PAGE_SIZE, CT_SERVER_URL};
use miette::{Context, IntoDiagnostic, Result};

use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
};

/// A newtype wrapper around assert_cmd::Command so that we can define custom methods.
/// For convenience it has a Deref impl that allows us to call assert_cmd methods
#[derive(Debug)]
pub struct Command(assert_cmd::Command);

impl Command {
    pub fn cargo_bin<S: AsRef<str>>(name: S) -> Self {
        assert_cmd::Command::cargo_bin(name)
            .into_diagnostic()
            .map(Self::from_assert_cmd)
            .expect("Couldn't find binary to test")
    }

    pub fn from_assert_cmd(cmd: assert_cmd::Command) -> Self {
        Self(cmd).default_env()
    }

    pub fn as_assert_cmd(&self) -> &assert_cmd::Command {
        &self.0
    }

    pub fn from_std(cmd: std::process::Command) -> Self {
        Self(assert_cmd::Command::from_std(cmd)).default_env()
    }

    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.0.env(key, value);
        self
    }

    pub fn page_size(&mut self, page_size: usize) -> &mut Self {
        self.env(CT_REST_PAGE_SIZE, page_size.to_string())
    }

    pub fn rest_debug(&mut self) -> &mut Self {
        self.env(CT_REST_DEBUG, "true")
    }

    // Apply default environment variables
    fn default_env(mut self) -> Self {
        self.env("NO_COLOR", "1");
        self
    }

    // Set environment variables to restrict CLI to offline usage only
    pub fn offline_env(&mut self) -> &mut Self {
        // Explicitly clear the API key so an individual dev's personal config isn't used for tests.
        self.env(CT_API_KEY, "");
        // Explicitly set the server to a bogus value that a server cannot to
        self.env(CT_SERVER_URL, "http://0.0.0.0:0");
        self
    }
}

/// Auto deref references to inner assert_cmd::Command type
impl Deref for Command {
    type Target = assert_cmd::Command;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Auto deref mutable references to inner assert_cmd::Command type
impl DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<assert_cmd::Command> for Command {
    fn from(cmd: assert_cmd::Command) -> Self {
        Self::from_assert_cmd(cmd)
    }
}

impl From<std::process::Command> for Command {
    fn from(cmd: std::process::Command) -> Self {
        Self::from_std(cmd)
    }
}

impl From<Command> for assert_cmd::Command {
    fn from(cmd: Command) -> Self {
        cmd.0
    }
}

/// Run a command from a string (used by cloudtruth! macro)
pub fn run_cloudtruth_cmd(bin_path: String, args: String) -> Result<Command> {
    // Use shlex to escape special characters in the binary path
    // also escapes backslashes in Windows path names
    let escaped_bin_path = shlex::quote(&bin_path);
    commandspec::commandify(format!("{escaped_bin_path} {args}"))
        .map(Command::from_std)
        .map_err(|e| e.compat())
        .into_diagnostic()
        .wrap_err_with(|| format!("Invalid command: {escaped_bin_path} {args}"))
}

/// Attempts to find the cloudtruth binary to test.
/// If not found via environment variables, will try to locate a binary with the given name in the current target directory
pub fn cli_bin_path<S: AsRef<str>>(name: S) -> String {
    std::env::var("NEXTEST_BIN_EXE_cloudtruth")
        .ok()
        .or(option_env!("CARGO_BIN_EXE_cloudtruth").map(String::from))
        .unwrap_or_else(|| assert_cmd::cargo::cargo_bin(name).display().to_string())
}
