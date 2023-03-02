use anyhow::*;
use once_cell::sync::Lazy;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::templates::HelpTextTemplate;

use super::{collect_file_errors, Cli};

macro_rules! help_text_path {
    ($($path:expr),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/help/", $($path),*)
    };
}

#[cfg(not(target_os = "windows"))]
const BIN_NAME: &str = "cloudtruth";
#[cfg(target_os = "windows")]
const BIN_NAME: &str = "cloudtruth.exe";

static HELP_TEXT_DIR: Lazy<PathBuf> =
    Lazy::new(|| Path::new(help_text_path!()).canonicalize().unwrap());

impl Cli {
    pub fn generate_help_text(&self) -> Result<()> {
        self.walk_cli(BIN_NAME)
    }

    fn walk_cli(&self, cmd_name: &str) -> Result<()> {
        self.mkdir(HELP_TEXT_DIR.as_path())?;
        self.walk_cli_inner(cmd_name, "")
    }

    fn walk_cli_inner(&self, cmd_name: &str, cmd_args: &str) -> Result<()> {
        let template = HelpTextTemplate::from_cmd(cmd_name, cmd_args)?;
        if self.verbose {
            println!("{} {}", template.cmd_name, template.cmd_args);
        }
        let path = HELP_TEXT_DIR.join(template.file_name());
        let file = self.open_output_file(path.as_path())?;
        template.write_md(file).with_context(|| { format!(
            "Error while rendering help text template {template_name:?} into {path:?}. {template:?}",
            template_name = template.file_name(),
        )})?;
        let results = template.subcommands().par_bridge().map(|subcommand| {
            let args = if cmd_args.is_empty() {
                Cow::from(subcommand)
            } else {
                Cow::from(format!("{cmd_args} {subcommand}"))
            };
            self.walk_cli_inner(cmd_name, &args)
        });
        collect_file_errors(
            anyhow!("Multiple errors when writing to help text files"),
            results.filter_map(Result::err).collect(),
        )
    }
}
