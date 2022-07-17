use clap::Shell;
use std::env;
use std::fs;
use std::io::{ErrorKind, Result};
use std::path::PathBuf;

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn main() -> Result<()> {
    if cfg!(any(
        feature = "bash",
        feature = "elvish",
        feature = "fish",
        feature = "powershell",
        feature = "zsh"
    )) {
        // OUT_DIR is set by Cargo and is where all generated files should go:
        // https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or(ErrorKind::NotFound)?);
        let completions_dir = out_dir.join("completions");
        fs::create_dir_all(&completions_dir)?;
        let mut app = cli::build_cli();
        if cfg!(bash) {
            app.gen_completions(cli::binary_name(), Shell::Bash, &completions_dir);
        }
        if cfg!(elvish) {
            app.gen_completions(cli::binary_name(), Shell::Elvish, &completions_dir);
        }
        if cfg!(fish) {
            app.gen_completions(cli::binary_name(), Shell::Fish, &completions_dir);
        }
        if cfg!(powershell) {
            app.gen_completions(cli::binary_name(), Shell::PowerShell, &completions_dir);
        }
        if cfg!(zsh) {
            app.gen_completions(cli::binary_name(), Shell::Zsh, &completions_dir);
        }
    }
    Ok(())
}
