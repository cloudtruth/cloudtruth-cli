use std::{collections::BTreeMap, fs::File};

use anyhow::*;
use maplit::btreemap;
use peeking_take_while::PeekableExt;
use serde::Serialize;

use crate::{
    config::{InstallType, ReleaseBuild, ReleaseTest, RunnerOs},
    matrices::{ReleaseBuildMatrix, ReleaseTestIncludes, ReleaseTestMatrix},
    matrices_base_path,
};

pub trait MatrixGenerator<'i> {
    type Key: Ord + Eq + Copy;
    type Input: 'i;
    type Matrix: Serialize;

    fn sort_key(conf: &Self::Input) -> Self::Key;

    fn file_map(&self) -> &BTreeMap<Self::Key, File>;

    fn generate_matrix<I: IntoIterator<Item = &'i Self::Input>>(&self, items: I) -> Self::Matrix;

    fn generate_from_config(&self, config: &'i mut [Self::Input]) -> Result<()> {
        config.sort_by_key(Self::sort_key);
        let mut tests_iter = config.iter().peekable();
        let mut errors = self
            .file_map()
            .iter()
            .filter_map(|(&key, file)| {
                let tests_for_install_type = tests_iter
                    .by_ref()
                    .peeking_take_while(|i| Self::sort_key(i) == key);
                let matrix = self.generate_matrix(tests_for_install_type);
                serde_json::to_writer(file, &matrix).err()
            })
            .peekable();
        if errors.peek().is_none() {
            Ok(())
        } else {
            Err(anyhow!(
                "Write errors: {}",
                errors
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }
    }
}

pub struct MatrixFiles<Key>(BTreeMap<Key, File>);

pub type BuildMatrixFiles = MatrixFiles<RunnerOs>;
pub type TestMatrixFiles = MatrixFiles<InstallType>;

impl BuildMatrixFiles {
    const LINUX_PATH: &'static str = concat!(matrices_base_path!(), "/build_release_linux.json");
    const MACOS_PATH: &'static str = concat!(matrices_base_path!(), "/build_release_macos.json");
    const WINDOWS_PATH: &'static str =
        concat!(matrices_base_path!(), "/build_release_windows.json");

    pub fn open_all() -> Result<BuildMatrixFiles> {
        Ok(MatrixFiles(btreemap![
            RunnerOs::UbuntuLatest => File::create(Self::LINUX_PATH)?,
            RunnerOs::MacosLatest => File::create(Self::MACOS_PATH)?,
            RunnerOs::WindowsLatest => File::create(Self::WINDOWS_PATH)?,
        ]))
    }
}

impl<'i> MatrixGenerator<'i> for BuildMatrixFiles {
    type Key = RunnerOs;

    type Input = ReleaseBuild;

    type Matrix = ReleaseBuildMatrix<'i>;

    fn sort_key(conf: &Self::Input) -> Self::Key {
        conf.runner
    }

    fn file_map(&self) -> &BTreeMap<Self::Key, File> {
        &self.0
    }

    fn generate_matrix<I: IntoIterator<Item = &'i Self::Input>>(&self, items: I) -> Self::Matrix {
        ReleaseBuildMatrix {
            target: items.into_iter().map(|i| i.target.as_str()).collect(),
        }
    }
}

impl TestMatrixFiles {
    const SHELL_PATH: &'static str = concat!(matrices_base_path!(), "/test_release_shell.json");
    const POWERSHELL_PATH: &'static str =
        concat!(matrices_base_path!(), "/test_release_powershell.json");
    const DOCKER_PATH: &'static str = concat!(matrices_base_path!(), "/test_release_docker.json");

    pub fn open_all() -> Result<TestMatrixFiles> {
        Ok(MatrixFiles(btreemap![
            InstallType::Shell => File::create(Self::SHELL_PATH)?,
            InstallType::PowerShell => File::create(Self::POWERSHELL_PATH)?,
            InstallType::Docker => File::create(Self::DOCKER_PATH)?,
        ]))
    }
}

impl<'i> MatrixGenerator<'i> for TestMatrixFiles {
    type Key = InstallType;
    type Input = ReleaseTest;
    type Matrix = ReleaseTestMatrix<'i>;

    fn sort_key(t: &ReleaseTest) -> InstallType {
        t.install_type
    }

    fn file_map(&self) -> &BTreeMap<Self::Key, File> {
        &self.0
    }

    fn generate_matrix<I: IntoIterator<Item = &'i ReleaseTest>>(
        &self,
        items: I,
    ) -> ReleaseTestMatrix<'i> {
        let mut matrix = ReleaseTestMatrix {
            os: Vec::new(),
            includes: Vec::new(),
        };
        for &ReleaseTest {
            os, ref versions, ..
        } in items
        {
            matrix.os.push(os);
            matrix
                .includes
                .extend(versions.iter().map(|version| ReleaseTestIncludes {
                    os,
                    runner: RunnerOs::from_test_os(os),
                    version,
                }))
        }
        matrix
    }
}
