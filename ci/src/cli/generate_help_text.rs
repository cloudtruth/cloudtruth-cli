use anyhow::{anyhow, Context, Result};
use futures::{Stream, StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use std::{fs, path::PathBuf, pin::Pin, sync::Arc};

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
        self.walk_cli(BIN_NAME).await
    }

    async fn walk_cli(&self, cmd_name: &'static str) -> Result<()> {
        self.mkdir(HELP_TEXT_DIR.as_path())?;
        let results: Vec<Result<()>> = self
            .walk_cli_subcommands(cmd_name, Arc::new(String::new()))
            .await
            .and_then(|template| self.write_template(template))
            .collect()
            .await;
        collect_file_errors(
            anyhow!("Multiple errors when fetching command help text"),
            results.into_iter().filter_map(Result::err).collect(),
        )
    }

    async fn walk_cli_subcommands<'a>(
        &'a self,
        cmd_name: &'static str,
        cmd_args: Arc<String>,
    ) -> Pin<Box<dyn Stream<Item = Result<HelpTextTemplate<'static>>> + 'a>> {
        match HelpTextTemplate::from_cmd_async(cmd_name, cmd_args.clone()).await {
            Err(err) => Box::pin(stream::once(Err(err))),
            Ok(template) => {
                let subcommands: Vec<String> = template.subcommands().map(String::from).collect();
                Box::pin(
                    stream::once(Ok(template)).chain(
                        stream::iter(subcommands)
                            .then(move |subcommand| {
                                let args = if cmd_args.is_empty() {
                                    subcommand
                                } else {
                                    format!("{cmd_args} {subcommand}")
                                };
                                self.walk_cli_subcommands(cmd_name, Arc::new(args))
                            })
                            .flatten(),
                    ),
                )
            }
        }
    }

    async fn write_template(&self, template: HelpTextTemplate<'static>) -> Result<()> {
        if self.verbose {
            println!("{} {}", template.cmd_name, template.cmd_args);
        }
        let path = HELP_TEXT_DIR.join(template.file_name());
        let file = self.open_output_file(path.as_path())?;
        template.write_md_async(file).await.with_context(|| { format!(
            "Error while rendering help text template {template_name:?} into {path:?}. {template:?}",
            template_name = template.file_name(),
        )})
    }
}
