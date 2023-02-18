mod config;
mod matrices;
mod matrix_generator;
mod templates;

use anyhow::*;
use askama::Template;
use clap::Parser;
use std::io::prelude::*;
use std::{
    fmt::Debug,
    fs::{DirBuilder, File},
    path::Path,
};

use config::*;
use matrix_generator::*;
use templates::*;

macro_rules! matrix_path {
    ($($path:literal),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/actions-matrices/", $($path),*)
    };
}

macro_rules! docker_path {
    ($($path:literal),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/docker/", $($path),*)
    };
}

macro_rules! config_yaml_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.yaml")
    };
}

const CONFIG_YAML: &str = include_str!(config_yaml_path!());

/// Helper for opening generated output files
pub fn open_file<P: Debug + AsRef<Path>>(path: P, verbose: bool) -> Result<File> {
    if verbose {
        println!("Writing to {path:?}");
    }
    Ok(File::create(path)?)
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    pretty: bool,
    #[arg(short, long)]
    verbose: bool,
}

fn generate_dockerfiles(cli: &Cli, config: &Config) -> Result<()> {
    let docker_base_path = Path::new(docker_path!());
    DirBuilder::new().recursive(true).create(docker_base_path)?;
    for template in config
        .release_tests
        .iter()
        .filter(|t| t.install_type == InstallType::Docker)
        .flat_map(DockerTemplate::from_config)
    {
        let path =
            docker_base_path.join(format!("Dockerfile.{}-{}", template.os, template.version));
        let mut file = open_file(path, cli.verbose)?;
        file.write_all(template.render()?.as_bytes())?;
    }
    Ok(())
}

fn generate_actions_matrices<'a: 'b, 'b>(cli: &Cli, config: &'a mut Config<'b>) -> Result<()> {
    DirBuilder::new().recursive(true).create(matrix_path!())?;
    let build_writer = BuildMatrixWriter::from_config(&mut config.release_builds);
    let test_writer = TestMatrixWriter::from_config(&mut config.release_tests);
    let build_release_file = open_file(matrix_path!("build_release.json"), cli.verbose)?;
    let test_release_file = open_file(matrix_path!("test_release.json"), cli.verbose)?;
    if cli.pretty {
        build_writer.write_json_pretty(build_release_file)?;
        test_writer.write_json_pretty(test_release_file)?;
    } else {
        build_writer.write_json(build_release_file)?;
        test_writer.write_json(test_release_file)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config: Config = serde_yaml::from_str(CONFIG_YAML)?;
    generate_dockerfiles(&cli, &config)?;
    generate_actions_matrices(&cli, &mut config)?;
    Ok(())
}
