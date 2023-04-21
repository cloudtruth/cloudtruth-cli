use cloudtruth_config::{CT_API_KEY, CT_REST_DEBUG, CT_REST_PAGE_SIZE, CT_SERVER_URL};
use miette::{miette, Result};
use once_cell::sync::OnceCell;
use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// A newtype wrapper around assert_cmd::Command so that we can define custom methods.
/// For convenience it has a Deref impl that allows us to call assert_cmd methods
#[derive(Debug)]
pub struct Command(assert_cmd::Command);

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self::from_assert_cmd(assert_cmd::Command::new(program))
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

/// Create a Command from a shell=like command line (used by cloudtruth! macro)
pub fn from_cmd_args<P: AsRef<Path>>(bin_path: P, args: String) -> Result<Command> {
    let bin_path = bin_path.as_ref();
    if args.trim().is_empty() {
        Ok(std::process::Command::new(bin_path).into())
    } else {
        let args = shlex::split(&args)
            .ok_or_else(|| miette!("Unable to parse command line arguments {:?}", args))?;
        let mut cmd = Command::new(bin_path);
        cmd.args(args);
        Ok(cmd)
    }
}

/// On Widows, we need to handle arguments differently
// #[cfg(windows)]÷
// pub fn from_cmd_args<P: AsRef<Path>>(bin_path: P, args: String) -> Result<Command> {
//     use std::os::windows::process::CommandExt;
//     let mut cmd = std::process::Command::new("cmd");
//     let cmd_line = format!("{} {}", bin_path.as_ref().to_string_lossy(), args);
//     // cmd.raw_arg(format!("/C {cmd_line}"));
//     Ok(Command::from_std(cmd))
// }

/// Attempts to find the CLI binary to test.
/// If not found via environment variables, will try to locate a binary with the given name in the current target directory
/// This logic runs once and then the result is cached for subsequent calls.
pub fn cli_bin_path<S: AsRef<str>>(name: S) -> &'static Path {
    static CLI_BIN_PATH: OnceCell<PathBuf> = OnceCell::new();
    CLI_BIN_PATH
        .get_or_init(|| {
            // try to find binary in target directory
            let bin_path = dunce::canonicalize(cargo_bin_path(name.as_ref()))
                .expect("Unable to canonicalize CLI path");
            println!("Found CLI binary at: {}", bin_path.display());
            bin_path
        })
        .as_ref()
}

/// Attempts to find the CLI binary in the cargo target directory.
fn cargo_bin_path<S: AsRef<str>>(name: S) -> PathBuf {
    std::env::var_os("NEXTEST_BIN_EXE_cloudtruth")
        .map(PathBuf::from)
        .or(option_env!("CARGO_BIN_EXE_cloudtruth").map(PathBuf::from))
        .unwrap_or_else(|| assert_cmd::cargo::cargo_bin(name.as_ref()))
}
