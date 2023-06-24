use std::{fmt::Display, io::Write, path::Path};

use anyhow::*;
use serde::Serialize;

use super::{collect_file_errors, Cli};
use crate::cicd_dir;
use crate::config::Config;
use crate::json::{ReleaseBuildMatrix, ReleaseTestMatrix};

/// Default base path for GH matrix outputs
macro_rules! matrix_path {
    ($($path:expr),*) => {
        concat!(cicd_dir!(),"/gha-matrices/", $($path),*)
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

    fn write_matrix<P: AsRef<Path>, V: Serialize + Display>(
        &self,
        output_dir: P,
        matrix_name: &str,
        json_value: V,
    ) -> Result<()> {
        if self.verbose {
            println!("Generated matrices for {matrix_name:?}: {matrix_name}");
        }
        let path = Path::new(output_dir.as_ref()).join(format!("{matrix_name}.json"));
        let file = self.open_output_file(path.as_path())?;
        self.write_json(file, &json_value)
            .with_context(|| format!("Error while serializing GHA matrix to {path:?}"))
    }

    pub fn generate_actions_matrices<'a: 'b, 'b>(&self, config: &'a Config<'b>) -> Result<()> {
        let matrix_dir = Path::new(matrix_path!()).canonicalize()?;
        self.mkdir(&matrix_dir)?;
        let results = vec![
            self.write_matrix(
                &matrix_dir,
                "release-builds",
                ReleaseBuildMatrix::from_iter(&config.release_builds),
            ),
            self.write_matrix(
                &matrix_dir,
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
