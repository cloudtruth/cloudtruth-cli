use cloudtruth_restapi::models::{AwsPullTaskStep, AwsPushTaskStep, TaskStep};

pub struct TaskStepDetails {
    pub url: String,
    pub id: String,
    pub provider: String,
    pub success: bool,
    pub detail: String,
    pub task_name: String,

    pub project_name: String,
    pub environment_name: String,
    pub parameter_name: String,
    pub venue_name: String,

    pub created_at: String,
    pub modified_at: String,
}

impl TaskStepDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "url" => self.url.clone(),
            "success" => self.success.to_string(),
            "result" => format!(
                "{} {}",
                if self.success { "SUCCESS" } else { "FAILED" },
                self.detail
            ),
            "detail" => self.detail.clone(),
            "task-name" => self.task_name.clone(),
            "project" => self.project_name.clone(),
            "environment" => self.environment_name.clone(),
            "parameter" => self.parameter_name.clone(),
            "venue-name" => self.venue_name.clone(),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl From<&AwsPushTaskStep> for TaskStepDetails {
    fn from(api: &AwsPushTaskStep) -> Self {
        let detail = if api.success {
            api.success_detail.clone().unwrap_or_default()
        } else {
            api.error_detail.clone().unwrap_or_default()
        };
        Self {
            url: api.url.clone(),
            id: api.id.clone(),
            provider: "aws".to_string(),
            success: api.success,
            detail,
            task_name: "".to_string(), // to be filled in later
            project_name: api.project_name.clone().unwrap_or_default(),
            environment_name: api.environment_name.clone().unwrap_or_default(),
            parameter_name: api.parameter_name.clone().unwrap_or_default(),
            venue_name: api.venue_name.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

impl From<&AwsPullTaskStep> for TaskStepDetails {
    fn from(api: &AwsPullTaskStep) -> Self {
        let detail = if api.success {
            api.success_detail.clone().unwrap_or_default()
        } else {
            api.error_detail.clone().unwrap_or_default()
        };
        Self {
            url: api.url.clone(),
            id: api.id.clone(),
            provider: "aws".to_string(),
            success: api.success,
            detail,
            task_name: "".to_string(), // to be filled in later
            project_name: api.project_name.clone().unwrap_or_default(),
            environment_name: api.environment_name.clone().unwrap_or_default(),
            parameter_name: api.parameter_name.clone().unwrap_or_default(),
            venue_name: api.venue_name.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

impl From<&TaskStep> for TaskStepDetails {
    fn from(api: &TaskStep) -> Self {
        let detail = if api.success {
            api.success_detail.clone().unwrap_or_default()
        } else {
            api.error_detail.clone().unwrap_or_default()
        };
        Self {
            url: api.url.clone(),
            id: api.id.clone(),
            provider: "".to_string(), // to be filled in later?
            success: api.success,
            detail,
            task_name: "".to_string(), // to be filled in later
            project_name: api.project_name.clone().unwrap_or_default(),
            environment_name: api.environment_name.clone().unwrap_or_default(),
            parameter_name: api.parameter_name.clone().unwrap_or_default(),
            venue_name: api.venue_name.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
