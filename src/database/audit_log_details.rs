use cloudtruth_restapi::models::AuditTrail;

#[derive(Clone, Debug)]
pub struct AuditLogDetails {
    pub id: String,
    pub action: String,
    pub object_id: String,
    pub object_name: String,
    pub object_type: String,
    pub timestamp: String,
    pub user: String,
}

impl From<&AuditTrail> for AuditLogDetails {
    fn from(api: &AuditTrail) -> Self {
        let user = &api.user;
        Self {
            id: api.id.clone(),
            action: api.action.clone(),
            object_id: api.object_id.clone(),
            object_name: api.object_name.clone(),
            object_type: api.object_type.clone(),
            timestamp: api.timestamp.clone(),
            user: user
                .name
                .clone()
                .unwrap_or_else(|| user.email.clone().unwrap_or_else(|| user.id.clone())),
        }
    }
}
