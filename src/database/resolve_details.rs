pub struct ResolvedDetails {
    env_name: String,
    env_id: String,
    proj_name: String,
    proj_id: String,
}

impl ResolvedDetails {
    pub fn new(env_name: String, env_id: String, proj_name: String, proj_id: String) -> Self {
        Self {
            env_name,
            env_id,
            proj_name,
            proj_id,
        }
    }

    pub fn environment_display_name(&self) -> &str {
        &self.env_name
    }

    pub fn project_display_name(&self) -> &str {
        &self.proj_name
    }

    pub fn project_id(&self) -> &str {
        &self.proj_id
    }

    pub fn environment_id(&self) -> &str {
        &self.env_id
    }
}
