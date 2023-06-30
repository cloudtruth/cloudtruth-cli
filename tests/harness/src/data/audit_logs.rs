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
    pub time: String,
    #[serde(rename = "Type")]
    pub object_type: String,
    pub user: String,
}

/* Use an extension trait to extend the assert_cmd::Assert type with a methods for getting parsed audit log entries */
pub trait GetAuditLogEntriesExt {
    fn get_audit_log_entries(&self) -> AuditLogEntries;
}

impl GetAuditLogEntriesExt for assert_cmd::assert::Assert {
    fn get_audit_log_entries(&self) -> AuditLogEntries {
        let out = &self.get_output().stdout;
        if out.starts_with(b"No audit log entries") {
            return AuditLogEntries(Vec::new());
        }
        let value: AuditLogRoot = serde_json::from_slice(out).expect("Invalid audit log JSON");
        value.audit_logs
    }
}
