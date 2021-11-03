use crate::database::task_detail::TaskDetail;
use cloudtruth_restapi::models::AwsPush;

#[derive(Clone, Debug)]
pub struct PushDetails {
    pub id: String,
    pub url: String,

    pub name: String,
    pub description: String,
    pub provider: String,
    pub resource: String,

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

impl PushDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "url" => self.url.clone(),
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "provider" => self.provider.clone(),
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

impl From<&AwsPush> for PushDetails {
    fn from(api: &AwsPush) -> Self {
        let task_detail = if let Some(task) = &api.latest_task {
            TaskDetail::from(&**task)
        } else {
            TaskDetail::default()
        };

        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            description: api.description.clone().unwrap_or_default(),
            provider: "aws".to_string(),
            resource: api.resource.clone(),
            project_urls: api.projects.clone(),
            project_names: vec![], // filled in later
            tag_urls: api.tags.clone(),
            tag_names: vec![], // filled in later
            last_task: task_detail,
            region: api.region.to_string(),
            service: api.service.to_string(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
