use cloudtruth_restapi::models::ImportParameter;

pub struct ImportDetails {
    pub project_name: String,
    pub project_id: String,
    pub environment_name: String,
    pub environment_id: String,
    pub parameter_name: String,
    pub parameter_id: String,
    pub secret: bool,
    pub value: String,
    pub value_id: String,
    pub action: String,
    pub created_at: String,
    pub modified_at: String,
}

impl ImportDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.parameter_name.clone(),
            "value" => self.value.clone(),
            "project" => self.project_name.clone(),
            "environment" => self.environment_name.clone(),
            "secret" => format!("{}", self.secret),
            "action" => self.action.clone(),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{property_name}'"),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl From<&ImportParameter> for ImportDetails {
    fn from(api: &ImportParameter) -> Self {
        Self {
            project_name: api.project_name.clone(),
            project_id: api.project_id.clone().unwrap_or_default(),
            environment_name: api.environment_name.clone(),
            environment_id: api.environment_id.clone().unwrap_or_default(),
            parameter_name: api.parameter_name.clone(),
            parameter_id: api.parameter_id.clone().unwrap_or_default(),
            secret: api.secret.unwrap_or_default(),
            value: api.value.clone(),
            value_id: api.value_id.clone().unwrap_or_default(),
            action: api.action.clone(),
            created_at: api.created_at.clone().unwrap_or_default(),
            modified_at: api.modified_at.clone().unwrap_or_default(),
        }
    }
}
