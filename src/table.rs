use color_eyre::eyre::ErrReport;
use csv::Error as CsvError;
use prettytable::Row as PrettyRow;
use prettytable::Table as PrettyTable;
use prettytable::{format, Attr, Cell};
use std::fmt;
use std::fmt::Formatter;
use std::io::stdout;

pub type Row = Vec<String>;

pub struct Table {
    header: Option<Row>,
    rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub enum TableError {
    UnhandledFormatError(String),
    CsvError(String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            TableError::UnhandledFormatError(format) => {
                write!(f, "Unhandled format: {}", format)
            }
            TableError::CsvError(details) => {
                write!(f, "CSV error: {}", details)
            }
        }
    }
}

// NOTE: CsvError does not implement the Clone trait, so just pass through a String
impl From<CsvError> for TableError {
    fn from(err: CsvError) -> Self {
        TableError::CsvError(err.to_string())
    }
}

impl From<TableError> for ErrReport {
    fn from(err: TableError) -> Self {
        ErrReport::msg(err.to_string())
    }
}

/// Convenience function to convert a Vec<&str> to a Row (aka Vec<String>).
pub fn to_row(vec: &[&str]) -> Row {
    vec.iter().map(|v| v.to_string()).collect()
}

impl Table {
    pub fn new() -> Self {
        Self {
            header: None,
            rows: Vec::new(),
        }
    }

    /// Adds the data for the header row of the table.
    pub fn set_header(&mut self, items: &[&str]) {
        let titles = to_row(items);
        self.header = Some(titles);
    }

    /// Add a Row to the table
    pub fn add_row(&mut self, row: Row) -> &mut Row {
        self.rows.push(row);
        let n = self.rows.len();
        &mut self.rows[n - 1]
    }

    /// Used to create a "basic" PrettyTable.
    ///
    /// The PrettyTable implements output for a couple different types. This provides a common means
    /// to create and populate the PrettyTable.
    fn make_prettytable(&self) -> PrettyTable {
        let mut table = PrettyTable::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        if let Some(header) = &self.header {
            let mut hdr_row = PrettyRow::new(Vec::new());
            for s in header {
                hdr_row.add_cell(Cell::new(s.as_str()).with_style(Attr::Bold))
            }
            table.set_titles(hdr_row);
        }
        for r in &self.rows {
            let mut pretty_row: PrettyRow = PrettyRow::new(Vec::new());
            for v in r {
                pretty_row.add_cell(Cell::new(v.as_str()));
            }
            table.add_row(pretty_row);
        }
        table
    }

    /// Renders the PrettyTable standard format.
    fn render_table(&self) -> Result<(), TableError> {
        let table = self.make_prettytable();
        table.printstd();
        Ok(())
    }

    /// Renders the CSV output, using PrettyTable.
    fn render_csv(&self) -> Result<(), TableError> {
        let table = self.make_prettytable();
        table.to_csv(stdout())?;
        Ok(())
    }

    /// Generic function to render the table in the `format` specified means.
    pub fn render(&self, format: &str) -> Result<(), TableError> {
        match format {
            "table" => self.render_table(),
            "csv" => self.render_csv(),
            _ => Err(TableError::UnhandledFormatError(format.to_string())),
        }
    }
}
