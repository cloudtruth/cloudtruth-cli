use clap::Shell;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

pub fn binary_name() -> String {
    option_env!("CARGO_BIN_NAME")
        .unwrap_or("cloudtruth")
        .to_string()
}

fn main() {
    // OUT_DIR is set by Cargo and is where all generated files should go:
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let mut completions_dir = PathBuf::from(&out_dir);
    completions_dir.push("completions");

    fs::create_dir_all(&completions_dir).unwrap();

    let mut app = cli::build_cli();
    app.gen_completions(binary_name(), Shell::Bash, &completions_dir);
    app.gen_completions(binary_name(), Shell::Elvish, &completions_dir);
    app.gen_completions(binary_name(), Shell::Fish, &completions_dir);
    app.gen_completions(binary_name(), Shell::PowerShell, &completions_dir);
    app.gen_completions(binary_name(), Shell::Zsh, &completions_dir);
}
