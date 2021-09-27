use cloudtruth_restapi::models::{Tag, TagReadUsage};
use once_cell::sync::OnceCell;

static DEFAULT_USAGE_VALUE: OnceCell<TagReadUsage> = OnceCell::new();

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
        let usage = match api.usage {
            Some(ref u) => u,
            _ => DEFAULT_USAGE_VALUE.get_or_init(|| TagReadUsage {
                last_read: None,
                last_read_by: "".to_string(),
                total_reads: 0,
            }),
        };
        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            description: api.description.clone().unwrap_or_default(),
            timestamp: api.clone().timestamp,
            last_use_user: usage.last_read_by.clone(),
            last_use_time: usage.last_read.clone().unwrap_or_default(),
            total_reads: usage.total_reads,
        }
    }
}
