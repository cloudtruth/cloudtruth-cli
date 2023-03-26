use anyhow::{anyhow, Context, Result};
use futures::{Stream, StreamExt};
use once_cell::sync::Lazy;
use std::{fs, path::PathBuf, pin::Pin};
use tokio_stream::{self as stream};

use crate::templates::HelpTextTemplate;

use super::{collect_file_errors, Cli};

macro_rules! help_text_path {
    ($($path:expr),*) => {
        concat!("../examples/help-text/", $($path),*)
    };
}

const BIN_NAME: &str = "cloudtruth";

static HELP_TEXT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    fs::canonicalize(help_text_path!())
        .context(help_text_path!())
        .unwrap()
});

impl Cli {
    pub async fn generate_help_text(&self) -> Result<()> {
        println!("{:?}", std::env::current_dir()?);
        self.walk_cli(BIN_NAME).await
    }

    async fn walk_cli(&self, cmd_name: &str) -> Result<()> {
        self.mkdir(HELP_TEXT_DIR.as_path())?;
        let results: Vec<Result<()>> = self.walk_cli_inner(cmd_name, "".to_owned()).collect().await;
        collect_file_errors(
            anyhow!("Multiple errors when writing to help text files"),
            results.into_iter().filter_map(Result::err).collect(),
        )
    }

    fn walk_cli_inner<'a>(
        &'a self,
        cmd_name: &'a str,
        cmd_args: String,
    ) -> Pin<Box<dyn Stream<Item = Result<()>> + 'a>> {
        match self.write_template(cmd_name, &cmd_args) {
            Err(err) => Box::pin(stream::once(Err(err))),
            Ok(template) => {
                let subcommands: Vec<String> = template.subcommands().map(String::from).collect();
                Box::pin(stream::iter(subcommands).flat_map(move |subcommand| {
                    let args = if cmd_args.is_empty() {
                        subcommand
                    } else {
                        format!("{cmd_args} {subcommand}")
                    };
                    self.walk_cli_inner(cmd_name, args)
                }))
            }
        }
    }

    fn write_template<'a>(
        &'a self,
        cmd_name: &'a str,
        cmd_args: &'a str,
    ) -> Result<HelpTextTemplate> {
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
        Ok(template)
    }
}
