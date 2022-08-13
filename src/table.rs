use color_eyre::eyre::ErrReport;
use csv::Error as CsvError;
use prettytable::Row as PrettyRow;
use prettytable::Table as PrettyTable;
use prettytable::{format, Attr, Cell};
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::io::Error as IoError;
use std::io::{stdout, Write};
use std::{cmp, fmt};

pub type Row = Vec<String>;

/// These types are used for formatting output data.
///
/// `BTreeMap` is used to get consistent order in JSON/YAML output.
type OutputItem = BTreeMap<String, String>;
type OutputList = Vec<OutputItem>;
type OutputData = BTreeMap<String, OutputList>;

#[derive(Debug)]
pub struct Table {
    object_type: String,
    header: Option<Row>,
    rows: Vec<Row>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError {
    UnhandledFormat(String),
    CsvError(String),
    JsonError(String),
    WriteError(String),
    YamlError(String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            TableError::UnhandledFormat(format) => {
                write!(f, "Unhandled format: {}", format)
            }
            TableError::CsvError(details) => {
                write!(f, "CSV error: {}", details)
            }
            TableError::JsonError(details) => {
                write!(f, "JSON error: {}", details)
            }
            TableError::WriteError(details) => {
                write!(f, "IO error: {}:", details)
            }
            TableError::YamlError(details) => {
                write!(f, "YAML error: {}", details)
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

// NOTE: IoError does not implement the Clone trait, so just pass through a String
impl From<IoError> for TableError {
    fn from(err: IoError) -> Self {
        TableError::WriteError(err.to_string())
    }
}

// NOTE: JsonError does not implement the Clone trait, so just pass through a String
impl From<JsonError> for TableError {
    fn from(err: JsonError) -> Self {
        TableError::JsonError(err.to_string())
    }
}

// NOTE: YamlError does not implement the Clone trait, so just pass through a String
impl From<YamlError> for TableError {
    fn from(err: YamlError) -> Self {
        TableError::YamlError(err.to_string())
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
    pub fn new(object_type: &str) -> Self {
        Self {
            object_type: object_type.to_string(),
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

    /// Find the longest row
    fn find_max_row_length(&self) -> usize {
        let mut len: usize = 0;
        for r in self.rows.iter() {
            len = cmp::max(len, r.len());
        }
        len
    }

    /// Get a vector of the header strings. This replaces empty strings with `Field X` since JSON
    /// format with a blank key is a bad practice.
    fn get_non_blank_headers(&self) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        let mut original = Row::new();
        if let Some(header) = &self.header {
            original = header.clone();
        }
        for column in 0..self.find_max_row_length() {
            let h = &original.get(column);
            if h.is_none() || h.unwrap().is_empty() {
                results.push(format!("Field {}", column + 1));
            } else {
                results.push(h.unwrap().clone());
            }
        }
        results
    }

    /// Creates a list of maps that both JSON and YAML can easily serialize.
    fn create_output_data(&self) -> OutputData {
        let mut list = OutputList::new();
        let headers = self.get_non_blank_headers();
        for row in &self.rows {
            let mut item = OutputItem::new();
            for (column, hdr_val) in headers.iter().enumerate() {
                item.insert(hdr_val.clone(), row.get(column).unwrap().clone());
            }
            list.push(item)
        }

        let mut data = OutputData::new();
        data.insert(self.object_type.clone(), list);
        data
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

    /// Writes the bulk of the JSON data to the provided output buffer.
    fn render_json_out<T: Write + ?Sized>(&self, out: &mut T) -> Result<(), TableError> {
        let data = self.create_output_data();
        serde_json::to_writer_pretty(out, &data)?;
        Ok(())
    }

    /// Renders the JSON output
    fn render_json(&self) -> Result<(), TableError> {
        self.render_json_out(&mut stdout())?;
        println!();
        Ok(())
    }

    /// Writes the bulk of the YAML data to the provided output buffer.
    fn render_yaml_out<T: Write>(&self, out: &mut T) -> Result<(), TableError> {
        let data = self.create_output_data();
        serde_yaml::to_writer(out, &data)?;
        Ok(())
    }

    /// Renders the YAML output
    fn render_yaml(&self) -> Result<(), TableError> {
        self.render_yaml_out(&mut stdout())?;
        Ok(())
    }

    /// Generic function to render the table in the `format` specified means.
    pub fn render(&self, format: &str) -> Result<(), TableError> {
        match format {
            "table" => self.render_table(),
            "csv" => self.render_csv(),
            "json" => self.render_json(),
            "yaml" => self.render_yaml(),
            _ => Err(TableError::UnhandledFormat(format.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::StringWriter;

    fn create_basic_table() -> Table {
        let mut table = Table::new("my_table_type");
        table.set_header(&["Column A", "Column 2", "Column III"]);
        table.add_row(to_row(&["A", "B", "c"]));
        table.add_row(to_row(&["1", "1024", "65535"]));
        table.add_row(to_row(&["Washington", "Adams", "Jefferson"]));
        table
    }

    #[test]
    fn test_unkown_format() {
        let table = create_basic_table();
        assert_eq!(
            table.render("foo").unwrap_err(),
            TableError::UnhandledFormat("foo".to_string())
        );
    }

    #[test]
    fn test_basic_json() {
        let mut writer = StringWriter::new();
        let table = create_basic_table();
        table.render_json_out(&mut writer).unwrap();
        let actual = serde_json::from_str::<serde_json::Value>(writer.as_string()).unwrap();
        let expected = serde_json::from_str::<serde_json::Value>(
            r#"{
  "my_table_type": [
    {
      "Column A": "A",
      "Column 2": "B",
      "Column III": "c"
    },
    {
      "Column A": "1",
      "Column 2": "1024",
      "Column III": "65535"
    },
    {
      "Column A": "Washington",
      "Column 2": "Adams",
      "Column III": "Jefferson"
    }
  ]
}"#,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_basic_yaml() {
        let mut writer = StringWriter::new();
        let table = create_basic_table();
        table.render_yaml_out(&mut writer).unwrap();
        let actual = serde_yaml::from_str::<serde_yaml::Value>(writer.as_string()).unwrap();
        let expected = serde_yaml::from_str::<serde_yaml::Value>(
            r#"---
my_table_type:
  - Column 2: B
    Column A: A
    Column III: c
  - Column 2: "1024"
    Column A: "1"
    Column III: "65535"
  - Column 2: Adams
    Column A: Washington
    Column III: Jefferson"#,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
}
