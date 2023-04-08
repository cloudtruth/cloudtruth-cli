use commandspec::CommandArg;
use miette::{IntoDiagnostic, Result};

use std::{
    env,
    ffi::OsStr,
    ops::{Deref, DerefMut},
};

const CLOUDTRUTH_REST_PAGE_SIZE: &str = "CLOUDTRUTH_REST_PAGE_SIZE";
const CLOUDTRUTH_REST_DEBUG: &str = "CLOUDTRUTH_REST_DEBUG";

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
        Self(cmd)
    }

    pub fn as_assert_cmd(&self) -> &assert_cmd::Command {
        &self.0
    }

    pub fn from_std(cmd: std::process::Command) -> Self {
        Self(assert_cmd::Command::from_std(cmd))
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
        self.env(CLOUDTRUTH_REST_PAGE_SIZE, page_size.to_string())
    }

    pub fn rest_debug(&mut self) -> &mut Self {
        self.env(CLOUDTRUTH_REST_DEBUG, "true")
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
pub fn commandify(cmd: String) -> Result<Command> {
    commandspec::commandify(cmd)
        .map(Command::from_std)
        .map_err(|e| e.compat())
        .into_diagnostic()
}

/// Process command argument (used by cloudtruth! macro)
pub fn command_arg<'a, A>(arg: &'a A) -> CommandArg
where
    CommandArg: From<&'a A>,
{
    CommandArg::from(arg)
}

/// Fetches the path to binary for this integration test. Will panic if not found.
pub fn bin_path() -> String {
    println!("{:?}", env::var("CARGO_BIN_EXE_cloudtruth"));
    env::var("CARGO_BIN_EXE_cloudtruth").expect("Could not find cloudtruth binary")
}
