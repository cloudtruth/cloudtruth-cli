mod cleanup;
mod generate_actions_matrices;
mod generate_dockerfiles;
mod generate_help_text;

use anyhow::*;
use clap::Parser;
use once_cell::sync::OnceCell;
use std::{
    fs::{create_dir, File},
    io::Read,
    path::Path,
};

use crate::config::Config;

#[macro_export]
macro_rules! workspace_dir {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/..")
    };
}

#[macro_export]
macro_rules! cicd_dir {
    () => {
        concat!($crate::workspace_dir!(), "/cicd")
    };
}

/// Default path to config file
#[macro_export]
macro_rules! cicd_config_yaml_path {
    () => {
        concat!($crate::cicd_dir!(), "/config.yaml")
    };
}

#[derive(clap::Parser, Clone, Debug)]
pub struct Cli {
    #[arg(long, short, global(true), help = "pretty-print output")]
    pub pretty: bool,
    #[arg(long, short, global(true), help = "verbose logging")]
    pub verbose: bool,
    #[command(subcommand)]
    pub task: TaskCommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum TaskCommand {
    #[command(
        about = "Bulk Data Cleanup",
        disable_help_flag = true,
        disable_help_subcommand = true
    )]
    Cleanup {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Generate Dockerfiles")]
    GenerateDocker,
    #[command(about = "Generate GitHub Actions job matrix data")]
    GenerateGhaMatrices,
    #[command(about = "Generate test cases for CLI help text")]
    GenerateHelpText,
}

impl Cli {
    pub fn get_cicd_config(&self) -> Result<&Config> {
        static CONFIG_YAML: OnceCell<String> = OnceCell::new();
        static CONFIG: OnceCell<Config> = OnceCell::new();
        CONFIG.get_or_try_init(|| {
            let yaml = CONFIG_YAML.get_or_try_init(|| {
                let config_yaml_path = Path::new(cicd_config_yaml_path!()).canonicalize()?;
                let mut buf = String::new();
                self.open_input_file(config_yaml_path)?
                    .read_to_string(&mut buf)
                    .context("Error serializing config YAML")?;
                Ok(buf)
            });
            Ok(serde_yaml::from_str(yaml?)?)
        })
    }
    #[allow(dead_code)]
    pub fn show_help() {
        use clap::CommandFactory;
        Self::command().print_help().unwrap();
    }
    fn open_input_file<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        if self.verbose {
            println!("Reading {}", path.as_ref().display());
        }
        File::open(path.as_ref())
            .with_context(|| format!("Unable to open file for reading: {:?}", path.as_ref()))
    }

    /// Helper for opening generated output files
    fn open_output_file<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        if self.verbose {
            println!("Writing {}", path.as_ref().display());
        }
        File::create(path.as_ref())
            .with_context(|| format!("Unable to open file for writing: {:?}", path.as_ref()))
    }
    fn mkdir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if !path.as_ref().is_dir() {
            create_dir(path.as_ref())
                .with_context(|| format!("Unable to create directory: {:?}", path.as_ref()))?;
            if self.verbose {
                println!("Created directory {}", path.as_ref().display())
            }
        } else if self.verbose {
            println!("Found directory {}", path.as_ref().display());
        }
        Ok(())
    }

    async fn run_task(&self) -> Result<()> {
        match &self.task {
            TaskCommand::Cleanup { args } => self.cleanup(args),
            TaskCommand::GenerateDocker => self.generate_dockerfiles(self.get_cicd_config()?).await,
            TaskCommand::GenerateGhaMatrices => {
                self.generate_actions_matrices(self.get_cicd_config()?)
            }
            TaskCommand::GenerateHelpText => self.generate_help_text().await,
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run_task().await
}

// collects and reports errors
pub fn collect_file_errors(aggregate_err: Error, mut errors: Vec<Error>) -> Result<()>
where
{
    match errors.len() {
        0 => Ok(()),
        1 => Err(errors.remove(0)),
        _ => {
            for err in errors.into_iter() {
                eprintln!("{err:#}");
            }
            Err(aggregate_err)
        }
    }
}
