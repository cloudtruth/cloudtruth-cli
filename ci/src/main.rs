mod config;
mod docker_template;
mod generate_actions_matrices;
mod generate_dockerfiles;
mod generate_help_text;
mod gh_matrix;
mod help_text_template;

use anyhow::*;
use clap::Parser;

use once_cell::sync::OnceCell;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::{create_dir, File},
    io::Read,
    path::Path,
};

use config::*;

/// Default path to config file
macro_rules! config_yaml_path {
    () => {
        "./config.yaml"
    };
}

#[derive(clap::Parser, Clone, Debug)]
pub struct Cli {
    #[arg(long, short, global(true), help = "pretty-print output")]
    pub pretty: bool,
    #[arg(long, short, global(true), help = "verbose logging")]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Commands {
    #[command(about = "Generate CI files")]
    Generate {
        #[arg(required = true)]
        actions: Vec<GenerateActions>,
    },
}

#[derive(clap::ValueEnum, Hash, PartialEq, Eq, Clone, Debug)]
pub enum GenerateActions {
    #[value(help = "Generate Dockerfiles")]
    Docker,
    #[value(help = "Generate GHA matrix files")]
    GhaMatrices,
    #[value(help = "Generate Dockerfiles")]
    HelpText,
}

impl Cli {
    pub fn get_config(&self) -> Result<&Config> {
        static CONFIG_YAML: OnceCell<String> = OnceCell::new();
        static CONFIG: OnceCell<Config> = OnceCell::new();
        CONFIG.get_or_try_init(|| {
            let yaml = CONFIG_YAML.get_or_try_init(|| {
                let config_yaml_path = Path::new(config_yaml_path!());
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let results = match &cli.command {
        Commands::Generate { actions } => HashSet::<&GenerateActions>::from_iter(actions)
            .into_par_iter()
            .map(|action| match action {
                GenerateActions::Docker => cli.generate_dockerfiles(cli.get_config()?),
                GenerateActions::GhaMatrices => cli.generate_actions_matrices(cli.get_config()?),
                GenerateActions::HelpText => cli.generate_help_text(),
            }),
    };
    collect_file_errors(
        anyhow!("Multiple file errors when generating CI files"),
        results.filter_map(Result::err).collect(),
    )
}

// collects and reports errors
fn collect_file_errors(aggregate_err: Error, mut errors: Vec<Error>) -> Result<()>
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
