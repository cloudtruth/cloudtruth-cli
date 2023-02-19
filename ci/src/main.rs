mod config;
mod docker_template;
mod gh_matrix;
mod gh_matrix_map;

use anyhow::*;

use clap::Parser;

use std::{
    fmt::Display,
    fs::{DirBuilder, File},
    io::Read,
    path::Path,
};

use config::*;
use docker_template::*;
use gh_matrix_map::*;

macro_rules! matrix_path {
    ($($path:literal),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/gh-actions/", $($path),*)
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

pub fn open_input_file<P: AsRef<Path>>(path: P, verbose: bool) -> Result<File> {
    if verbose {
        println!("Reading from {path}", path = path.as_ref().display());
    }
    Ok(File::open(path)?)
}

/// Helper for opening generated output files
pub fn open_output_file<P: AsRef<Path>>(path: P, verbose: bool) -> Result<File> {
    if verbose {
        println!("Writing to {path}", path = path.as_ref().display());
    }
    Ok(File::create(path)?)
}

pub fn display_matrix_map<K, M>(name: &str, map: &MatrixMap<K, M>)
where
    MatrixMap<K, M>: Display,
{
    print!("=== Generated matrices for {name} ===\n{map}");
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    pretty: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, help = "Build GitHub Actions matrices")]
    actions: bool,
    #[arg(long, help = "Build Dockerfiles")]
    docker: bool,
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
            let result = open_output_file(&path, cli.verbose)
                .and_then(|file| template.write_dockerfile(file));
            (path, result)
        });
    report_file_errors(results)
}

fn generate_actions_matrices<'a: 'b, 'b>(cli: &Cli, config: &'a mut Config<'b>) -> Result<()> {
    DirBuilder::new().recursive(true).create(matrix_path!())?;
    let build_map = BuildMatrixMap::from_config(&mut config.release_builds);
    if cli.verbose {
        display_matrix_map("build_release", &build_map);
    }
    let test_map = TestMatrixMap::from_config(&mut config.release_tests);
    if cli.verbose {
        display_matrix_map("test_release", &test_map)
    };
    let build_release_file = open_output_file(matrix_path!("build_release.json"), cli.verbose)?;
    let test_release_file = open_output_file(matrix_path!("test_release.json"), cli.verbose)?;
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
    let config_yaml_path = Path::new(config_yaml_path!());
    let mut config_yaml = String::new();
    open_input_file(config_yaml_path, cli.verbose)?.read_to_string(&mut config_yaml)?;
    let mut config: Config = serde_yaml::from_str(&config_yaml)?;
    if cli.docker || !cli.actions {
        generate_dockerfiles(&cli, &config)?;
    }
    if cli.actions || !cli.docker {
        generate_actions_matrices(&cli, &mut config)?;
    }
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
