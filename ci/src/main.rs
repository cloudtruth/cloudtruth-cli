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

const BUILD_RELEASE_PATH: &'static str = matrix_path!("build_release.json");

const TEST_RELEASE_PATH: &'static str = matrix_path!("test_release.json");

/// Helper for opening generated output files
pub fn open_file<P: Debug + AsRef<Path>>(path: P, verbose: bool) -> Result<File> {
    if verbose {
        println!("Writing to {:?}", path);
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config: Config = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/config.yaml"
    )))?;
    let docker_base_path = Path::new(docker_path!());
    DirBuilder::new().recursive(true).create(docker_base_path)?;
    for template in config
        .release_tests
        .iter()
        .filter(|t| t.install_type == InstallType::Docker)
        .flat_map(DockerTemplate::from_config)
    {
        let path = &format!("Dockerfile.{}-{}", template.os, template.version);
        let mut file = open_file(docker_base_path.join(path), cli.verbose)?;
        file.write_all(template.render()?.as_bytes())?;
    }
    DirBuilder::new()
        .recursive(true)
        .create(Path::new(matrix_path!()))?;
    let build_writer = BuildMatrixWriter::from_config(config.release_builds.as_mut_slice());
    let test_writer = TestMatrixWriter::from_config(config.release_tests.as_mut_slice());
    let build_release_file = open_file(BUILD_RELEASE_PATH, cli.verbose)?;
    let test_release_file = open_file(TEST_RELEASE_PATH, cli.verbose)?;
    if cli.pretty {
        build_writer.write_json_pretty(build_release_file)?;
        test_writer.write_json_pretty(test_release_file)?;
    } else {
        build_writer.write_json(build_release_file)?;
        test_writer.write_json(test_release_file)?;
    }
    Ok(())
}
