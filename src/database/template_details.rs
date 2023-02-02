use cloudtruth_restapi::models::Template;

#[derive(Debug)]
pub struct TemplateDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<&Template> for TemplateDetails {
    fn from(api_temp: &Template) -> Self {
        TemplateDetails {
            id: api_temp.id.clone(),
            name: api_temp.name.clone(),
            description: api_temp.description.clone().unwrap_or_default(),
            body: api_temp.body.clone().unwrap_or_default(),
            created_at: api_temp.created_at.clone(),
            modified_at: api_temp.modified_at.clone().unwrap_or_default(),
        }
    }
}
