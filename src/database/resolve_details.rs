use crate::config::DEFAULT_ENV_NAME;

pub struct ResolvedDetails {
    pub env_name: Option<String>,
    pub env_id: Option<String>,
    pub proj_name: Option<String>,
    pub proj_id: Option<String>,
}

impl ResolvedDetails {
    pub fn environment_display_name(&self) -> String {
        self.env_name
            .clone()
            .unwrap_or_else(|| DEFAULT_ENV_NAME.to_string())
    }

    pub fn project_display_name(&self) -> String {
        self.proj_name.clone().unwrap_or_default()
    }

    pub fn project_id(&self) -> &str {
        self.proj_id.as_deref().unwrap()
    }

    pub fn environment_id(&self) -> &str {
        self.env_id.as_deref().unwrap()
    }
}
