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
    CSV,
    JSON,
    YAML,
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
            "csv" => Ok(Self::CSV),
            "json" => Ok(Self::JSON),
            "yaml" => Ok(Self::YAML),
            _ => Err(Self::Error::UnhandledFormat(value.to_string())),
        }
    }
}

pub trait HasOutputProperties {
    fn get_property(&self, property_name: &str) -> Value;
}

pub struct OutputFormatterBuilder<'a> {
    output_name: &'a str,
    format_type: OutputFormatType,
    properties: Vec<String>,
    show_headers: bool,
}

impl<'a> OutputFormatterBuilder<'a> {
    pub fn new(output_name: &'a str) -> Self {
        OutputFormatterBuilder {
            output_name,
            format_type: OutputFormatType::default(),
            properties: Vec::new(),
            show_headers: true,
        }
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
            output_name: self.output_name.to_string(),
            show_headers: self.show_headers,
            properties: self.properties,
            objects: Vec::new(),
        }
    }
}

pub struct OutputFormatter {
    output_name: String,
    format_type: OutputFormatType,
    show_headers: bool,
    properties: Vec<String>,
    objects: Vec<IndexMap<String, Value>>,
}

impl OutputFormatter {
    pub fn add_object<O>(&mut self, object: &O) -> &mut Self
    where
        O: HasOutputProperties,
    {
        let mut map = IndexMap::new();
        self.properties.iter().for_each(|prop| {
            map.insert(prop.clone(), object.get_property(prop));
        });
        self.objects.push(map);
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
        for obj in &self.objects {
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
            OutputFormatType::CSV => self.render_csv(),
            OutputFormatType::JSON => self.render_json(),
            OutputFormatType::YAML => self.render_yaml(),
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
        let data = indexmap! {
            self.output_name.to_string() => &self.objects
        };
        serde_json::to_writer_pretty(out, &data)?;
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
        let data = indexmap! {
            self.output_name.to_string() => &self.objects
        };
        serde_yaml::to_writer(out, &data)?;
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
        Value::Array(vec) => vec.iter().map(value_to_table_string).join(", "),
        Value::Object(map) => map
            .iter()
            .map(|(key, val)| format!("({}={})", key, value_to_table_string(val)))
            .join(","),
        other => other.to_string(),
    }
}
