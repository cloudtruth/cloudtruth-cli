use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

use tempfile::NamedTempFile;

/// A file containing test data stored in a temp directory.
///
/// Provides convenient helpers and trait implementations for writing test cases involving files.
///
/// Automatically derefs to a File for convenience, and supports AsRef conversion to
/// both File and Path.
///
/// Display implementation prints the file path as a string, making it easy to reference the file path
/// in CLI tests.
#[derive(Debug, Display)]
#[display(fmt = "{}", "temp_file.path().display()")]
pub struct TestFile {
    temp_file: NamedTempFile,
}

impl TestFile {
    /// Creates an empty test file.
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            temp_file: NamedTempFile::new()?,
        })
    }

    /// Creates a new test file with the given contents.
    pub fn with_contents<S: AsRef<[u8]>>(contents: S) -> std::io::Result<Self> {
        let mut test_file = Self::new()?;
        test_file.temp_file.write_all(contents.as_ref())?;
        Ok(test_file)
    }

    /// The file path of the test file.
    pub fn path(&self) -> &Path {
        self.temp_file.path()
    }
}

impl Deref for TestFile {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        self.temp_file.as_file()
    }
}

impl DerefMut for TestFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.temp_file.as_file_mut()
    }
}

impl AsRef<File> for TestFile {
    fn as_ref(&self) -> &File {
        self.temp_file.as_file()
    }
}

impl AsMut<File> for TestFile {
    fn as_mut(&mut self) -> &mut File {
        self.temp_file.as_file_mut()
    }
}

impl From<TestFile> for File {
    fn from(val: TestFile) -> Self {
        val.temp_file.into_file()
    }
}

impl AsRef<Path> for TestFile {
    fn as_ref(&self) -> &Path {
        self.temp_file.path()
    }
}
