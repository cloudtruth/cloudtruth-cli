use cloudtruth_restapi::models::Environment;

#[derive(Debug)]
pub struct EnvironmentDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub parent_url: String,
    pub parent_name: String,
    pub created_at: String,
    pub modified_at: String,
}

/// Converts the OpenApi `Environment` reference into a CloudTruth `EnvironmentDetails` object.
///
/// The `parent_name` is filled in later, so it can be done with a map of URLs to names.
impl From<&Environment> for EnvironmentDetails {
    fn from(api_env: &Environment) -> Self {
        Self {
            id: api_env.id.clone(),
            url: api_env.url.clone(),
            name: api_env.name.clone(),
            description: api_env.description.clone().unwrap_or_default(),
            parent_url: api_env.parent.clone().unwrap_or_default(),
            parent_name: "".to_owned(),
            created_at: api_env.created_at.clone(),
            modified_at: api_env.modified_at.clone().unwrap_or_default(),
        }
    }
}
