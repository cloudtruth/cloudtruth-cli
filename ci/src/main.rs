mod config;
mod matrices;
mod matrix_generator;
mod templates;

use std::{fs::DirBuilder, path::Path};

use config::*;
use matrix_generator::*;

use anyhow::*;

macro_rules! matrices_base_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/actions-matrices")
    };
}
pub(crate) use matrices_base_path;

fn main() -> Result<()> {
    let mut config: Config = serde_yaml::from_str(include_str!("../config.yaml"))?;
    DirBuilder::new()
        .recursive(true)
        .create(Path::new(matrices_base_path!()))?;
    BuildMatrixFiles::open_all()?.generate_from_config(config.release_builds.as_mut_slice())?;
    TestMatrixFiles::open_all()?.generate_from_config(config.release_tests.as_mut_slice())?;
    Ok(())
}
