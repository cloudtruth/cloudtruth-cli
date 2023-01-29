use cloudtruth_restapi::models::Project;

#[derive(Clone, Debug)]
pub struct ProjectDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub parent_name: String,
    pub parent_url: String,
    pub parameter_name_pattern: String,
    pub created_at: String,
    pub modified_at: String,
}

/// Converts from the OpenApi `Project` model to the CloudTruth `ProjectDetails`
impl From<&Project> for ProjectDetails {
    fn from(api_proj: &Project) -> Self {
        Self {
            id: api_proj.id.clone(),
            url: api_proj.url.clone(),
            name: api_proj.name.clone(),
            parameter_name_pattern: api_proj.parameter_name_pattern.unwrap_or_default(),
            description: api_proj.description.clone().unwrap_or_default(),
            parent_url: api_proj.depends_on.clone().unwrap_or_default(),
            parent_name: "".to_string(),
            created_at: api_proj.created_at.clone(),
            modified_at: api_proj.modified_at.clone().unwrap_or_default(),
        }
    }
}
