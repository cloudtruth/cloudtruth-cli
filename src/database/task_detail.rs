use cloudtruth_restapi::models::{AwsPullTask, AwsPushTask};

#[derive(Clone, Debug)]
pub struct TaskDetail {
    pub id: String,
    pub url: String,
    pub reason: String,
    pub state: String,
    pub error_code: String,
    pub error_detail: String,
    pub created_at: String,
    pub modified_at: String,
}

impl TaskDetail {
    pub fn summary(&self) -> String {
        let mut out = self.reason.clone();
        if !self.state.is_empty() {
            if !out.is_empty() {
                out = format!("{}: {}", out, self.state);
            } else {
                out = self.state.clone();
            }
        }

        if self.state != "success" && !self.error_detail.is_empty() {
            if !out.is_empty() {
                out = format!("{} - {}", out, self.error_detail);
            } else {
                out = self.error_detail.clone();
            }
        }

        if !self.modified_at.is_empty() {
            if !out.is_empty() {
                out = format!("{} ({})", out, self.modified_at);
            } else {
                out = self.modified_at.clone();
            }
        }
        out
    }

    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "url" => self.url.clone(),
            "reason" => self.reason.clone(),
            "state" => self.state.clone(),
            "error-code" => self.error_code.clone(),
            "error-detail" => self.error_detail.clone(),
            "errors" => {
                if self.state == "success" {
                    "".to_string()
                } else if self.error_code.is_empty() {
                    self.error_detail.clone()
                } else if self.error_detail.is_empty() {
                    self.error_code.clone()
                } else {
                    format!("{}: {}", self.error_code, self.error_detail)
                }
            }
            "summary" => self.summary(),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl Default for TaskDetail {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            url: "".to_string(),
            reason: "".to_string(),
            state: "".to_string(),
            error_code: "".to_string(),
            error_detail: "".to_string(),
            created_at: "".to_string(),
            modified_at: "".to_string(),
        }
    }
}

impl From<&AwsPushTask> for TaskDetail {
    fn from(api: &AwsPushTask) -> Self {
        let state = if let Some(state_enum) = api.state.clone() {
            state_enum.to_string().to_lowercase()
        } else {
            "".to_string()
        };

        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            reason: api.reason.clone().unwrap_or_default(),
            state,
            error_code: api.error_code.clone().unwrap_or_default(),
            error_detail: api.error_detail.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

impl From<&AwsPullTask> for TaskDetail {
    fn from(api: &AwsPullTask) -> Self {
        let state = if let Some(state_enum) = api.state.clone() {
            state_enum.to_string().to_lowercase()
        } else {
            "".to_string()
        };

        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            reason: api.reason.clone().unwrap_or_default(),
            state,
            error_code: api.error_code.clone().unwrap_or_default(),
            error_detail: api.error_detail.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
