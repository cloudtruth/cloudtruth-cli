use cloudtruth_restapi::models::AuditTrailSummary;

#[derive(Clone, Debug)]
pub struct AuditLogSummary {
    pub total_records: i32,
    pub earliest: String,

    // these reflect policy
    pub max_days: i32,
    pub max_records: i32,
}

impl From<&AuditTrailSummary> for AuditLogSummary {
    fn from(api: &AuditTrailSummary) -> Self {
        Self {
            total_records: api.total,
            earliest: api.earliest.clone().unwrap_or_default(),
            max_days: api.max_days,
            max_records: api.max_records,
        }
    }
}
