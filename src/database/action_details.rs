use crate::database::task_detail::TaskDetail;
use cloudtruth_restapi::models::{AwsPull, AwsPush};

#[derive(Clone, Debug)]
pub struct ActionDetails {
    pub id: String,
    pub url: String,

    pub name: String,
    pub integration_name: String,
    pub description: String,
    pub provider: String,
    pub action_type: String,
    pub resource: String,
    pub dry_run: Option<bool>,
    pub flags: Vec<String>,

    // these are push specific
    pub project_urls: Vec<String>,
    pub project_names: Vec<String>,
    pub tag_urls: Vec<String>,
    pub tag_names: Vec<String>,

    pub last_task: TaskDetail,

    // these may be Amazon specific
    pub region: String,
    pub service: String,

    pub created_at: String,
    pub modified_at: String,
}

impl ActionDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "url" => self.url.clone(),
            "name" => self.name.clone(),
            "integration" => self.integration_name.clone(),
            "description" => self.description.clone(),
            "provider" => self.provider.clone(),
            "action" => self.action_type.clone(),
            "dry-run" => match self.dry_run {
                None => "none".to_string(),
                Some(b) => b.to_string(),
            },
            "flags" => {
                if self.flags.is_empty() {
                    "none".to_string()
                } else {
                    self.flags.join(", ")
                }
            }
            "resource" => self.resource.clone(),
            "project-urls" => self.project_urls.join(", "),
            "project-names" => self.project_names.join(", "),
            "tag-urls" => self.tag_urls.join(", "),
            "tag-names" => self.tag_names.join(", "),
            "task-info" => self.last_task.get_property("summary"),
            "task-time" => self.last_task.get_property("modified-at"),
            "task-state" => self.last_task.get_property("state"),
            "region" => self.region.clone(),
            "service" => self.service.clone(),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl From<&AwsPush> for ActionDetails {
    fn from(api: &AwsPush) -> Self {
        let task_detail = if let Some(task) = &api.latest_task {
            TaskDetail::from(&**task)
        } else {
            TaskDetail::default()
        };
        let mut flags: Vec<String> = vec![];
        if let Some(dry_run) = api.dry_run {
            if dry_run {
                flags.push("dry-run".to_string());
            }
        }
        if let Some(params) = api.include_parameters {
            if params {
                let mut flag_name = "parameters".to_string();
                if let Some(coerced) = api.coerce_parameters {
                    if coerced {
                        flag_name = "parameters-coerced".to_string();
                    }
                }
                flags.push(flag_name);
            }
        }
        if let Some(secrets) = api.include_secrets {
            if secrets {
                flags.push("secrets".to_string());
            }
        }
        if let Some(force_owner) = api.force {
            if force_owner {
                flags.push("no-check-owner".to_string());
            }
        }

        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            integration_name: "".to_string(), // filled in later
            description: api.description.clone().unwrap_or_default(),
            provider: "aws".to_string(),
            action_type: "push".to_string(),
            resource: api.resource.clone().unwrap_or_default(),
            project_urls: api.projects.clone(),
            project_names: vec![], // filled in later
            tag_urls: api.tags.clone(),
            tag_names: vec![], // filled in later
            dry_run: api.dry_run,
            flags,
            last_task: task_detail,
            region: match &api.region {
                Some(r) => r.to_string(),
                _ => "".to_string(),
            },
            service: match &api.service {
                Some(s) => s.to_string(),
                _ => "".to_string(),
            },
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

impl From<&AwsPull> for ActionDetails {
    fn from(api: &AwsPull) -> Self {
        let task_detail = if let Some(task) = &api.latest_task {
            TaskDetail::from(&**task)
        } else {
            TaskDetail::default()
        };
        let mut flags: Vec<String> = vec![];
        if let Some(dry_run) = api.dry_run {
            if dry_run {
                flags.push("dry-run".to_string());
            }
        }

        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            integration_name: "".to_string(), // filled in later
            description: api.description.clone().unwrap_or_default(),
            provider: "aws".to_string(),
            action_type: "pull".to_string(),
            dry_run: api.dry_run,
            flags,
            resource: api.resource.clone().unwrap_or_default(),
            project_urls: vec![],
            project_names: vec![], // filled in later
            tag_urls: vec![],
            tag_names: vec![], // filled in later
            last_task: task_detail,
            region: match &api.region {
                Some(r) => r.to_string(),
                _ => "".to_string(),
            },
            service: match &api.service {
                Some(s) => s.to_string(),
                _ => "".to_string(),
            },
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
