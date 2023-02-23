mod config;
mod docker_template;
mod gh_matrix;
mod gh_matrix_map;

use anyhow::*;
use clap::Parser;
use serde::Serialize;
use std::{
    fmt::Display,
    fs::{create_dir, File},
    io::{Read, Write},
    path::Path,
};

use config::*;
use docker_template::*;
use gh_matrix_map::*;

/// Default base path for GH matrix outputs
macro_rules! matrix_path {
    ($($path:expr),*) => {
        concat!("./gh-actions/", $($path),*)
    };
}

/// Default base path for docker outputs
macro_rules! docker_path {
    ($($path:expr),*) => {
        concat!("./docker/", $($path),*)
    };
}

/// Default path to config file
macro_rules! config_yaml_path {
    () => {
        "./config.yaml"
    };
}

#[derive(clap::Parser)]
struct Cli {
    #[arg(long, short)]
    pretty: bool,
    #[arg(long, short)]
    verbose: bool,
    #[arg(long, help = "Build GitHub Actions matrices")]
    actions: bool,
    #[arg(long, help = "Build Dockerfiles")]
    docker: bool,
}

impl Cli {
    pub fn show_help() {
        use clap::CommandFactory;
        Self::command().print_help().unwrap();
    }
    pub fn open_input_file<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        if self.verbose {
            println!("Reading {}", path.as_ref().display());
        }
        File::open(path.as_ref())
            .with_context(|| format!("Unable to open file for reading: {:?}", path.as_ref()))
    }

    /// Helper for opening generated output files
    pub fn open_output_file<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        if self.verbose {
            println!("Writing {}", path.as_ref().display());
        }
        File::create(path.as_ref())
            .with_context(|| format!("Unable to open file for writing: {:?}", path.as_ref()))
    }

    pub fn display_matrix_map<K, M>(&self, name: &str, map: &MatrixMap<K, M>)
    where
        MatrixMap<K, M>: Display,
    {
        print!("=== Generated matrices for {name} ===\n{map}");
    }

    pub fn mkdir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
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

    pub fn write_json<W: Write, T: ?Sized + Serialize>(&self, writer: W, value: &T) -> Result<()> {
        if self.pretty {
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
            let serializer = &mut serde_json::Serializer::with_formatter(writer, formatter);
            value.serialize(serializer)?;
        } else {
            serde_json::to_writer(writer, &value)?;
        }
        Ok(())
    }

    fn generate_actions_matrices<'a: 'b, 'b>(&self, config: &'a mut Config<'b>) -> Result<()> {
        self.mkdir(matrix_path!())?;
        let results = vec![
            self.write_matrix_map(
                "build-release",
                BuildMatrixMap::from_config(&mut config.release_builds),
            ),
            self.write_matrix_map(
                "test-release",
                TestMatrixMap::from_config(&mut config.release_tests),
            ),
        ];
        collect_file_errors(
            anyhow!("Multiple errors while writing GHA matrices"),
            results.into_iter().filter_map(Result::err).collect(),
        )
    }

    fn write_matrix_map<K, M>(&self, name: &str, map: MatrixMap<K, M>) -> Result<()>
    where
        MatrixMap<K, M>: Serialize + Display,
    {
        if self.verbose {
            self.display_matrix_map(name, &map);
        }
        let path = Path::new(matrix_path!()).join(format!("{name}.json"));
        let file = self.open_output_file(path.as_path())?;
        self.write_json(file, &map)
            .with_context(|| format!("Error while serializing GHA matrix to {path:?}"))
    }

    fn generate_dockerfiles(&self, config: &Config) -> Result<()> {
        let docker_base_path = Path::new(docker_path!());
        self.mkdir(docker_base_path)?;
        let results = DockerTemplate::iter_from_config(&config.release_tests).map(|template| {
            let path = docker_base_path.join(template.file_name());
            let file = self.open_output_file(path.as_path())?;
            template.write_dockerfile(file).with_context(|| {
                format!(
                    "Error while rendering template at {template_name:?} into {path:?}. {template:?}",
                    template_name = template.file_name(),
                )
            })
        });
        collect_file_errors(
            anyhow!("Multiple file errors when generating Dockerfiles"),
            results.filter_map(Result::err).collect(),
        )
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if !cli.actions && !cli.docker {
        Cli::show_help();
        eprintln!("ERROR: One of the following options is required: --actions, --docker");
        std::process::exit(1);
    }
    let config_yaml_path = Path::new(config_yaml_path!());
    let mut config_yaml = String::new();
    cli.open_input_file(config_yaml_path)?
        .read_to_string(&mut config_yaml)
        .context("Error serializing config YAML")?;
    let mut config: Config = serde_yaml::from_str(&config_yaml)?;
    let mut errors = Vec::new();
    if cli.docker {
        if let Err(err) = cli.generate_dockerfiles(&config) {
            errors.push(err);
        }
    }
    if cli.actions {
        if let Err(err) = cli.generate_actions_matrices(&mut config) {
            errors.push(err);
        }
    }
    collect_file_errors(
        anyhow!("Multiple file errors when generating CI files"),
        errors,
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
