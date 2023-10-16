use std::{
    env,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use backtrace::Backtrace;
use thiserror::Error;

use miette::{Diagnostic, NamedSource, Report, SourceOffset, SourceSpan};

#[derive(Debug, Error, Diagnostic)]
#[error("Test failure at {}:{}:{}", filename, line, col)]
#[diagnostic()]
/// A miette report for test case source code snippets
pub struct TestSourceSpan {
    filename: String,
    line: usize,
    col: usize,
    #[source_code]
    src: NamedSource,
    #[label("Test failure")]
    span: SourceSpan,
    #[related]
    related: Vec<Report>,
}

impl TestSourceSpan {
    /// Fetch miette source code and source span from given filename and line
    pub fn from_location(
        filename: String,
        line: usize,
        col: usize,
    ) -> std::io::Result<TestSourceSpan> {
        let mut file = File::open(&filename)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        let start_offset = SourceOffset::from_location(&source, line, col).offset();
        // find byte offset at end of line
        let end_offset = source[start_offset..]
            .lines()
            .next()
            .map(|line| start_offset + line.trim_end().len())
            .unwrap_or_else(|| source.trim_end().len());
        let span = (start_offset..end_offset).into();
        Ok(TestSourceSpan {
            src: NamedSource::new(&filename, source),
            span,
            filename,
            line,
            col,
            related: Vec::new(),
        })
    }

    /// Add an error to the list of related errors
    pub fn add_related<E: Into<Report>>(&mut self, err: E) {
        self.related.push(err.into());
    }

    /// Tries to find source information from backtrace.
    pub fn from_backtrace() -> io::Result<Option<Self>> {
        // A substring of test source file paths
        let test_path = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .or(env::current_dir())
            .unwrap();
        eprintln!("{test_path:?}");
        /* Go through backtrace in reverse order to get the top-level source snippet */
        for frame in Backtrace::new().frames().iter().rev() {
            for symbol in frame.symbols().iter() {
                if let Some(filename) = symbol.filename() {
                    eprintln!("{filename:?}");
                    if filename.starts_with(&test_path) {
                        if let Some(name) = symbol.name() {
                            /* skip attribute macros */
                            if 
                                cfg!(target_os = "windows") && name.to_string().contains("::closure$0")
                                || name.to_string().contains("::{{closure}}")
                                 {
                                continue;
                            }
                        }
                        eprintln!("{symbol:?}");
                        if let (Some(line), Some(col)) = (symbol.lineno(), symbol.colno()) {
                            return Ok(Some(Self::from_location(
                                filename.to_string_lossy().into(),
                                line as usize,
                                col as usize,
                            )?));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
