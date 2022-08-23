use indexmap::indexmap;
use indexmap::IndexMap;
use itertools::Itertools;
use prettytable::Row as PrettyRow;
use prettytable::Table as PrettyTable;
use prettytable::{format, Attr, Cell};
use serde_json::Value;
use std::convert::TryFrom;
use std::io::stdout;
use std::io::Write;

use crate::output_formatter_error::OutputFormatterError;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum OutputFormatType {
    Table,
    Csv,
    Json,
    Yaml,
}

impl Default for OutputFormatType {
    fn default() -> Self {
        Self::Table
    }
}

impl TryFrom<&str> for OutputFormatType {
    type Error = OutputFormatterError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "table" => Ok(Self::Table),
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            _ => Err(Self::Error::UnhandledFormat(value.to_string())),
        }
    }
}

pub trait HasOutputProperties {
    fn get_property(&self, property_name: &str) -> Value;
}

pub struct OutputFormatterBuilder {
    output_name: Option<String>,
    format_type: OutputFormatType,
    properties: Vec<String>,
    show_headers: bool,
}

impl OutputFormatterBuilder {
    pub fn new() -> Self {
        OutputFormatterBuilder {
            output_name: None,
            format_type: OutputFormatType::default(),
            properties: Vec::new(),
            show_headers: true,
        }
    }

    /* If specified, will wrap the output list in a top-level structure (for JSON and YAML only)  */
    pub fn output_name(&mut self, name: &str) -> &mut Self {
        self.output_name = Some(name.to_string());
        self
    }

    /* Whether or not to include headers in tabular output formats (Table, CSV). Default is true. */
    pub fn show_headers(&mut self, show_headers: bool) -> &mut Self {
        self.show_headers = show_headers;
        self
    }

    pub fn property_names<I>(&mut self, properties: I) -> &mut Self
    where
        I: IntoIterator<Item = String>,
    {
        self.properties.extend(properties);
        self
    }

    pub fn build(self) -> OutputFormatter {
        OutputFormatter {
            format_type: self.format_type,
            output_name: self.output_name,
            show_headers: self.show_headers,
            properties: self.properties,
            records: Vec::new(),
        }
    }
}

pub struct OutputFormatter {
    output_name: Option<String>,
    format_type: OutputFormatType,
    show_headers: bool,
    properties: Vec<String>,
    records: Vec<IndexMap<String, Value>>,
}

impl OutputFormatter {
    pub fn add_record<O>(&mut self, record: &O) -> &mut Self
    where
        O: HasOutputProperties,
    {
        let mut map = IndexMap::new();
        self.properties.iter().for_each(|prop| {
            map.insert(prop.clone(), record.get_property(prop));
        });
        self.records.push(map);
        self
    }

    /// Used to create a "basic" PrettyTable.
    ///
    /// The PrettyTable implements output for a couple different types. This provides a common means
    /// to create and populate the PrettyTable.
    fn make_prettytable(&self) -> Result<PrettyTable, OutputFormatterError> {
        let mut table = PrettyTable::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        if self.show_headers {
            table.set_titles(PrettyRow::new(
                self.properties
                    .iter()
                    .map(Cell::from)
                    .map(|c| c.with_style(Attr::Bold))
                    .collect::<Vec<Cell>>(),
            ));
        }
        for obj in &self.records {
            table.add_row(PrettyRow::new(
                self.properties
                    .iter()
                    .map(|prop| {
                        Cell::from(&obj.get(prop).map(value_to_table_string).unwrap_or_default())
                    })
                    .collect::<Vec<Cell>>(),
            ));
        }
        Ok(table)
    }

    pub fn render(&self) -> Result<(), OutputFormatterError> {
        match self.format_type {
            OutputFormatType::Table => self.render_table(),
            OutputFormatType::Csv => self.render_csv(),
            OutputFormatType::Json => self.render_json(),
            OutputFormatType::Yaml => self.render_yaml(),
        }
    }

    /// Renders the PrettyTable standard format.
    fn render_table(&self) -> Result<(), OutputFormatterError> {
        let table = self.make_prettytable()?;
        table.printstd();
        Ok(())
    }

    /// Renders the CSV output, using PrettyTable.
    fn render_csv(&self) -> Result<(), OutputFormatterError> {
        let table = self.make_prettytable()?;
        table.to_csv(stdout())?;
        Ok(())
    }

    /// Writes the bulk of the JSON data to the provided output buffer.
    fn render_json_out<T: Write + ?Sized>(&self, out: &mut T) -> Result<(), OutputFormatterError> {
        if let Some(name) = &self.output_name {
            let data = indexmap!(
                name => &self.records
            );
            serde_json::to_writer_pretty(out, &data)?
        } else {
            serde_json::to_writer_pretty(out, &self.records)?
        }
        Ok(())
    }

    /// Renders the JSON output
    fn render_json(&self) -> Result<(), OutputFormatterError> {
        self.render_json_out(&mut stdout())?;
        println!();
        Ok(())
    }

    /// Writes the bulk of the YAML data to the provided output buffer.
    fn render_yaml_out<T: Write>(&self, out: &mut T) -> Result<(), OutputFormatterError> {
        if let Some(name) = &self.output_name {
            let data = indexmap!(
                name => &self.records
            );
            serde_yaml::to_writer(out, &data)?
        } else {
            serde_yaml::to_writer(out, &self.records)?
        }
        Ok(())
    }

    /// Renders the YAML output
    fn render_yaml(&self) -> Result<(), OutputFormatterError> {
        self.render_yaml_out(&mut stdout())?;
        Ok(())
    }
}

fn value_to_table_string(val: &Value) -> String {
    match val {
        Value::String(str) => str.clone(),
        Value::Array(vec) => vec.iter().map(value_to_table_string).join(","),
        Value::Object(map) => map
            .iter()
            .map(|(key, val)| format!("({}={})", key, value_to_table_string(val)))
            .join(","),
        Value::Null => String::default(),
        other => other.to_string(),
    }
}
