use std::{fs::File, io::Read};

use thiserror::Error;

use miette::{Diagnostic, NamedSource, Report, SourceOffset, SourceSpan};

#[derive(Debug, Error, Diagnostic)]
#[error("Test failure")]
#[diagnostic()]
pub struct TestSourceSpan {
    filename: String,
    line: usize,
    col: usize,
    #[source_code]
    src: NamedSource,
    #[label("test failed here")]
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
        let offset = SourceOffset::from_location(&source, line, col).offset();
        let span = (offset..offset + 1).into();
        Ok(TestSourceSpan {
            src: NamedSource::new(&filename, source),
            span,
            filename,
            line,
            col,
            related: Vec::new(),
        })
    }

    pub fn add_related<E: Into<Report>>(&mut self, err: E) {
        self.related.push(err.into());
    }
}
