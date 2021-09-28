use cloudtruth_restapi::models::Tag;

#[derive(Debug)]
pub struct EnvironmentTag {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub timestamp: String,
    pub last_use_user: String,
    pub last_use_time: String,
    pub total_reads: i32,
}

impl From<&Tag> for EnvironmentTag {
    fn from(api: &Tag) -> Self {
        let usage = &api.usage;
        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            description: api.description.clone().unwrap_or_default(),
            timestamp: api.clone().timestamp,
            last_use_user: usage.last_read_by.clone().unwrap_or_default(),
            last_use_time: usage.last_read.clone().unwrap_or_default(),
            total_reads: usage.total_reads,
        }
    }
}
