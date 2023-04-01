use anyhow::*;
use std::path::Path;
use tokio_stream::{self as stream, StreamExt};

use crate::{config::Config, templates::DockerTemplate};

use super::{collect_file_errors, Cli};

/// Default base path for docker outputs
macro_rules! docker_path {
    ($($path:expr),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/docker/", $($path),*)
    };
}

impl Cli {
    pub async fn generate_dockerfiles(&self, config: &Config<'_>) -> Result<()> {
        let docker_base_path = Path::new(docker_path!());
        self.mkdir(docker_base_path)?;
        let results: Vec<Result<()>> = stream::iter(DockerTemplate::iter_from_config(&config.release_tests)).then(|template| async move {
            let path = docker_base_path.join(template.file_name());
            let file = self.open_output_file(path.as_path())?;
            template.write_dockerfile_async(file).await.with_context(|| {
                format!(
                    "Error while rendering template at {template_name:?} into {path:?}. {template:?}",
                    template_name = template.file_name(),
                )
            })
        }).collect().await;
        collect_file_errors(
            anyhow!("Multiple file errors when generating Dockerfiles"),
            results.into_iter().filter_map(Result::err).collect(),
        )
    }
}
