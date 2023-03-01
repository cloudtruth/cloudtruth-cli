use std::{env, io::Write, iter::once, path::PathBuf};

use anyhow::*;
use askama::Template;
use duct::cmd;

/// Template for generating the installation test Dockerfiles
#[derive(Debug, Template)]
#[template(path = "help-text.md", escape = "none")]
//#[template(print = "code")] //uncomment for debugging generated code
pub struct HelpTextTemplate<'a, 'b> {
    pub cmd_name: &'a str,
    pub cmd_args: &'b str,
    pub help_text: String,
}

impl<'a, 'b> HelpTextTemplate<'a, 'b> {
    pub fn from_cmd(cmd_name: &'a str, cmd_args: &'b str) -> Result<HelpTextTemplate<'a, 'b>> {
        let bin_path = cargo_bin_str(cmd_name).canonicalize()?;
        let cmd = cmd(
            bin_path.as_path(),
            cmd_args
                .split(' ')
                .filter(|s| !s.is_empty())
                .chain(once("--help")),
        );
        let mut help_text = cmd.read().with_context(|| {
            format!(
                "Error running command {bin_path} {cmd_args}",
                bin_path = bin_path.display()
            )
        })?;
        // strip platform-specific extensions from command name
        let base_cmd = cmd_name.replace(env::consts::EXE_SUFFIX, "");
        //add the trycmd matcher to match EXE_SUFFIX
        help_text = help_text.replace(cmd_name, &format!("{base_cmd}[EXE]"));
        Ok(HelpTextTemplate {
            cmd_name,
            cmd_args,
            help_text,
        })
    }

    pub fn subcommands(&self) -> impl Iterator<Item = &str> {
        self.help_text
            .lines()
            .skip_while(|line| !line.starts_with("SUBCOMMANDS"))
            .skip(1)
            .filter_map(|line| {
                line.strip_prefix("    ")?
                    .split(' ')
                    .next()
                    .filter(|&s| !s.is_empty() && s != "help")
            })
    }

    pub fn file_name(&self) -> String {
        if self.cmd_args.is_empty() {
            format!("{}.md", self.cmd_name)
        } else {
            format!("{}-{}.md", self.cmd_name, self.cmd_args.replace(' ', "-"))
        }
    }

    pub fn write_md<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(self.render()?.as_bytes())?;
        Ok(())
    }
}

// Adapted from
// https://docs.rs/assert_cmd/latest/src/assert_cmd/cargo.rs.html#203
fn cargo_bin_str(name: &str) -> PathBuf {
    target_dir().join(format!("debug/{}{}", name, env::consts::EXE_SUFFIX))
}

// Adapted from
// https://github.com/rust-lang/cargo/blob/485670b3983b52289a2f353d589c57fae2f60f82/tests/testsuite/support/mod.rs#L507
fn target_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap().canonicalize().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.pop();
    path
}
