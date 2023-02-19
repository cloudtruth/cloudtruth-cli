use itertools::Itertools;
use std::{collections::BTreeMap, fmt::Display, io::Write};

use anyhow::*;
use serde::Serialize;

use crate::{
    config::{InstallType, RunnerOs},
    gh_matrix::{ReleaseBuildMatrix, ReleaseTestMatrix},
};

/// Trait for specifying the sort key for partitioning config data into matricies
pub trait HasSortKey {
    type Key: Ord + Eq + Copy;

    fn sort_key(&self) -> Self::Key;
}

#[derive(Serialize, Debug)]
pub struct MatrixMap<Key, Matrix>(BTreeMap<Key, Matrix>);
pub type BuildMatrixMap<'c> = MatrixMap<RunnerOs, ReleaseBuildMatrix<'c>>;
pub type TestMatrixMap<'c> = MatrixMap<InstallType, ReleaseTestMatrix<'c>>;

impl<'c, Key, Matrix> MatrixMap<Key, Matrix> {
    /// Load from config. Need mutable reference to sort the data for partitioning.
    pub fn from_config<Conf>(confs: &'c mut [Conf]) -> Self
    where
        Key: Ord + Eq + Copy,
        Conf: HasSortKey<Key = Key>,
        Matrix: FromIterator<&'c Conf>,
    {
        confs.sort_by_key(Conf::sort_key);
        Self::from_config_sorted(confs)
    }

    /// Load from pre-sorted config
    pub fn from_config_sorted<Conf>(confs: &'c [Conf]) -> Self
    where
        Key: Ord + Eq + Copy,
        Conf: HasSortKey<Key = Key>,
        Matrix: FromIterator<&'c Conf>,
    {
        MatrixMap(
            confs
                .iter()
                .group_by(|c| c.sort_key())
                .into_iter()
                .map(|(key, group)| (key, Matrix::from_iter(group)))
                .collect(),
        )
    }

    pub fn write_json<W: Write>(&self, writer: W) -> Result<()>
    where
        Key: Serialize,
        Matrix: Serialize,
    {
        serde_json::to_writer(writer, &self)?;
        Ok(())
    }

    pub fn write_json_pretty<W: Write>(&self, writer: W) -> Result<()>
    where
        Key: Serialize,
        Matrix: Serialize,
    {
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
        let serializer = &mut serde_json::Serializer::with_formatter(writer, formatter);
        self.serialize(serializer)?;
        Ok(())
    }
}

impl<K, M> Display for MatrixMap<K, M>
where
    K: Display,
    M: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, matrix) in &self.0 {
            writeln!(f, "{key}: {matrix}")?;
        }
        std::fmt::Result::Ok(())
    }
}
