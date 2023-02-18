mod config;
mod matrices;
mod matrix_map;
mod templates;

use anyhow::*;

use clap::Parser;

use std::{
    fmt::Debug,
    fs::{DirBuilder, File},
    path::Path,
};

use config::*;
use matrix_map::*;
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
    let results = config
        .release_tests
        .iter()
        .filter(|t| t.install_type == InstallType::Docker)
        .flat_map(DockerTemplate::from_release_test_config)
        .map(|template| {
            let path = docker_base_path.join(template.file_name());
            let result =
                open_file(&path, cli.verbose).and_then(|file| template.write_dockerfile(file));
            (path, result)
        });
    report_file_errors(results)
}

fn generate_actions_matrices<'a: 'b, 'b>(cli: &Cli, config: &'a mut Config<'b>) -> Result<()> {
    DirBuilder::new().recursive(true).create(matrix_path!())?;
    let build_map = BuildMatrixMap::from_config(&mut config.release_builds);
    let test_map = TestMatrixMap::from_config(&mut config.release_tests);
    let build_release_file = open_file(matrix_path!("build_release.json"), cli.verbose)?;
    let test_release_file = open_file(matrix_path!("test_release.json"), cli.verbose)?;
    if cli.pretty {
        build_map.write_json_pretty(build_release_file)?;
        test_map.write_json_pretty(test_release_file)?;
    } else {
        build_map.write_json(build_release_file)?;
        test_map.write_json(test_release_file)?;
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

// collects and reports errors
fn report_file_errors<I, P, T>(results: I) -> Result<()>
where
    I: IntoIterator<Item = (P, Result<T>)>,
    P: AsRef<Path>,
{
    let mut err_paths = Vec::new();
    for (p, result) in results {
        if let Err(err) = result {
            let path_display = p.as_ref().display();
            eprintln!("Error: Could not write to {path_display}: {err}");
            err_paths.push(path_display.to_string());
        }
    }
    if err_paths.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Could not write to file(s): {}",
            err_paths.join(" ")
        ))
    }
}
