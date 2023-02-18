mod config;
mod matrices;
mod matrix_generator;
mod templates;

use anyhow::*;
use askama::Template;
use clap::Parser;
use std::io::prelude::*;
use std::{
    fs::{DirBuilder, File},
    path::Path,
};

use config::*;
use matrix_generator::*;
use templates::*;

macro_rules! matrices_base_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/actions-matrices")
    };
}

macro_rules! docker_base_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/docker")
    };
}

struct BuildMatrixFile;

impl BuildMatrixFile {
    const PATH: &'static str = concat!(matrices_base_path!(), "/build_release.json");
    pub fn open() -> Result<File> {
        Ok(File::create(Self::PATH)?)
    }
}

struct TestMatrixFile;

impl TestMatrixFile {
    const PATH: &'static str = concat!(matrices_base_path!(), "/test_release.json");
    pub fn open() -> Result<File> {
        Ok(File::create(Self::PATH)?)
    }
}

struct Dockerfile;

impl Dockerfile {
    const BASE_PATH: &'static str = docker_base_path!();
    pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
        Ok(File::create(Path::new(Self::BASE_PATH).join(path))?)
    }
}
#[derive(Parser)]
struct Cli {
    #[arg(long)]
    pretty: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config: Config = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/config.yaml"
    )))?;
    for template in config
        .release_tests
        .iter()
        .filter(|t| t.install_type == InstallType::Docker)
        .flat_map(DockerTemplate::from_config)
    {
        let mut file =
            Dockerfile::open(format!("Dockerfile.{}-{}", template.os, template.version))?;
        file.write_all(template.render()?.as_bytes())?;
    }
    DirBuilder::new()
        .recursive(true)
        .create(Path::new(matrices_base_path!()))?;
    let build_writer = BuildMatrixWriter::from_config(config.release_builds.as_mut_slice());
    let test_writer = TestMatrixWriter::from_config(config.release_tests.as_mut_slice());
    if cli.pretty {
        build_writer.write_json_pretty(BuildMatrixFile::open()?)?;
        test_writer.write_json_pretty(TestMatrixFile::open()?)?;
    } else {
        build_writer.write_json(BuildMatrixFile::open()?)?;
        test_writer.write_json(TestMatrixFile::open()?)?;
    }
    Ok(())
}
