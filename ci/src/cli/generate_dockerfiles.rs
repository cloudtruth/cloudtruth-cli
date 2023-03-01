use anyhow::*;
use rayon::prelude::*;
use std::path::Path;

use crate::{config::Config, templates::docker_template::DockerTemplate};

use super::{collect_file_errors, Cli};

/// Default base path for docker outputs
macro_rules! docker_path {
    ($($path:expr),*) => {
        concat!("./docker/", $($path),*)
    };
}

impl Cli {
    pub fn generate_dockerfiles(&self, config: &Config) -> Result<()> {
        let docker_base_path = Path::new(docker_path!());
        self.mkdir(docker_base_path)?;
        let results = DockerTemplate::iter_from_config(&config.release_tests).par_bridge().map(|template| {
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