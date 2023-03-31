use commandspec::CommandArg;
use miette::{IntoDiagnostic, Result};
use std::{
    env,
    ops::{Deref, DerefMut},
};
/// A newtype wrapper around assert_cmd::Command so that we can define custom methods.
/// For convenience it has a Deref impl that allows us to call assert_cmd methods
#[derive(Debug)]
pub struct Command(assert_cmd::Command);

impl Command {
    pub fn cargo_bin<S: AsRef<str>>(name: S) -> Result<Self> {
        assert_cmd::Command::cargo_bin(name)
            .into_diagnostic()
            .map(Self::from_assert_cmd)
    }

    pub fn from_assert_cmd(cmd: assert_cmd::Command) -> Self {
        Self(cmd)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_assert_cmd(self) -> assert_cmd::Command {
        self.0
    }

    pub fn from_std(cmd: std::process::Command) -> Self {
        Self(assert_cmd::Command::from_std(cmd))
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

impl From<Command> for assert_cmd::Command {
    fn from(val: Command) -> Self {
        val.to_assert_cmd()
    }
}

impl From<std::process::Command> for Command {
    fn from(cmd: std::process::Command) -> Self {
        Self::from_std(cmd)
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
pub fn command_arg<'a, T>(value: &'a T) -> CommandArg
where
    CommandArg: From<&'a T>,
{
    CommandArg::from(value)
}

/// Fetches the path to binary for this integration test. Will panic if not found.
pub fn bin_path() -> String {
    println!("{:?}", env::var("CARGO_BIN_EXE_cloudtruth"));
    env::var("CARGO_BIN_EXE_cloudtruth").expect("Could not find cloudtruth binary")
}
