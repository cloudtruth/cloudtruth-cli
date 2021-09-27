use cloudtruth_restapi::models::Project;

#[derive(Debug)]
pub struct ProjectDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub modified_at: String,
}

/// Converts from the OpenApi `Project` model to the CloudTruth `ProjectDetails`
impl From<&Project> for ProjectDetails {
    fn from(api_proj: &Project) -> Self {
        Self {
            id: api_proj.id.clone(),
            name: api_proj.name.clone(),
            description: api_proj.description.clone().unwrap_or_default(),
            created_at: api_proj.created_at.clone(),
            modified_at: api_proj.modified_at.clone(),
        }
    }
}
