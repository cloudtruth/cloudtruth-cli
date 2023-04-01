use std::{fmt::Display, io::Write, path::Path};

use anyhow::*;
use serde::Serialize;

use crate::config::Config;
use crate::json::{ReleaseBuildMatrix, ReleaseTestMatrix};

use super::{collect_file_errors, Cli};

/// Default base path for GH matrix outputs
macro_rules! matrix_path {
    ($($path:expr),*) => {
        concat!(env!("CARGO_MANIFEST_DIR"),"/gha-matrices/", $($path),*)
    };
}

impl Cli {
    fn write_json<W: Write, T: ?Sized + Serialize>(&self, writer: W, value: &T) -> Result<()> {
        if self.pretty {
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
            let serializer = &mut serde_json::Serializer::with_formatter(writer, formatter);
            value.serialize(serializer)?;
        } else {
            serde_json::to_writer(writer, &value)?;
        }
        Ok(())
    }

    fn write_matrix<V: Serialize + Display>(&self, name: &str, value: V) -> Result<()> {
        if self.verbose {
            println!("Generated matrices for {name:?}: {value}");
        }
        let path = Path::new(matrix_path!()).join(format!("{name}.json"));
        let file = self.open_output_file(path.as_path())?;
        self.write_json(file, &value)
            .with_context(|| format!("Error while serializing GHA matrix to {path:?}"))
    }

    pub fn generate_actions_matrices<'a: 'b, 'b>(&self, config: &'a Config<'b>) -> Result<()> {
        self.mkdir(matrix_path!())?;
        let results = vec![
            self.write_matrix(
                "release-builds",
                ReleaseBuildMatrix::from_iter(&config.release_builds),
            ),
            self.write_matrix(
                "release-tests",
                ReleaseTestMatrix::from_iter(&config.release_tests),
            ),
        ];
        collect_file_errors(
            anyhow!("Multiple errors while writing GHA matrices"),
            results.into_iter().filter_map(Result::err).collect(),
        )
    }
}
