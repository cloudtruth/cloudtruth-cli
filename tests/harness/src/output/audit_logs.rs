use chrono::{DateTime, Utc};
use core::{fmt::Debug, str::FromStr};
use serde::Deserialize;

/* Root object of `audit-logs ls` command */
#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogRoot {
    #[serde(rename = "audit-logs")]
    pub audit_logs: AuditLogEntries,
}

/* An array of Audit Log entry objects */
#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogEntries(pub Vec<AuditLogEntry>);

impl std::ops::Deref for AuditLogEntries {
    type Target = Vec<AuditLogEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AuditLogEntries {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Helper functions for searching a list of audit log entries */
impl AuditLogEntries {
    /* Find entries whose "Action" field matches the string */
    pub fn find_by_action(&self, action: impl AsRef<str>) -> impl Iterator<Item = &AuditLogEntry> {
        self.0
            .iter()
            .filter(move |entry| entry.action == action.as_ref())
    }
    /* Find entries whose "Type" field matches the string */
    pub fn find_by_type(
        &self,
        object_type: impl AsRef<str>,
    ) -> impl Iterator<Item = &AuditLogEntry> {
        self.0
            .iter()
            .filter(move |entry| entry.object_type == object_type.as_ref())
    }

    pub fn get_create_delete_count(
        &self,
        object_type: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> (usize, usize) {
        self.iter()
            .filter(move |entry| {
                entry.object_type == object_type.as_ref() && entry.object_name == name.as_ref()
            })
            .fold((0, 0), |(create_count, delete_count), entry| {
                match entry.action.as_str() {
                    "create" => (create_count + 1, delete_count),
                    "delete" => (create_count, delete_count + 1),
                    _ => (create_count, delete_count),
                }
            })
    }
}

/* A parsed audit log entry object */
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuditLogEntry {
    pub action: String,
    #[serde(rename = "Object Name")]
    pub object_name: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "Type")]
    pub object_type: String,
    pub user: String,
}

/* Use an extension trait to extend the assert_cmd::Assert type with a methods for getting parsed audit log entries */
pub trait GetAuditLogEntriesExt {
    fn parse_audit_log_json(&self) -> AuditLogEntries;
}

impl GetAuditLogEntriesExt for assert_cmd::assert::Assert {
    fn parse_audit_log_json(&self) -> AuditLogEntries {
        let out = &self.get_output().stdout;
        if out.starts_with(b"No audit log entries") {
            return AuditLogEntries(Vec::new());
        }
        let value: AuditLogRoot = serde_json::from_slice(out).expect("Invalid audit log JSON");
        value.audit_logs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogSummary {
    earliest_record: DateTime<Utc>,
    record_count: usize,
    maximum_records: usize,
    maximum_days: usize,
}

impl AuditLogSummary {
    pub fn parse(output: &[u8]) -> AuditLogSummary {
        let output = String::from_utf8_lossy(output);
        AuditLogSummary {
            earliest_record: Self::parse_value(&output, "Earliest record"),
            record_count: Self::parse_value(&output, "Record count"),
            maximum_records: Self::parse_value(&output, "Maximum records"),
            maximum_days: Self::parse_value(&output, "Maximum days"),
        }
    }

    fn parse_value<T>(output: &str, label: &str) -> T
    where
        T: FromStr,
        T::Err: Debug,
    {
        output
            .lines()
            .find_map(|line| line.trim_start().strip_prefix(&format!("{label}:")))
            .unwrap_or_else(|| panic!("Could not find \"{label}\" in audit summary"))
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("Could not parse \"{label}\" in audit summary"))
    }

    pub fn earliest_record(&self) -> &DateTime<Utc> {
        &self.earliest_record
    }

    pub fn record_count(&self) -> usize {
        self.record_count
    }

    pub fn maximum_records(&self) -> usize {
        self.maximum_records
    }

    pub fn maximum_days(&self) -> usize {
        self.maximum_days
    }
}
