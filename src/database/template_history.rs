use crate::database::HistoryAction;
use cloudtruth_restapi::models::TemplateTimelineEntry;

#[derive(Debug, Clone)]
pub struct TemplateHistory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub change_type: HistoryAction,
    pub modified_at: String,
    pub user_name: String,
}

impl From<&TemplateTimelineEntry> for TemplateHistory {
    fn from(api: &TemplateTimelineEntry) -> Self {
        let api_template = api.history_template.clone().unwrap_or_default();
        TemplateHistory {
            id: api_template.id.clone(),
            name: api_template.name.clone(),
            description: api_template.description.clone().unwrap_or_default(),
            body: api_template.body.clone().unwrap_or_default(),
            change_type: HistoryAction::from(*api.history_type.clone().unwrap_or_default()),
            modified_at: api.modified_at.clone().unwrap_or_default(),
            user_name: api.modified_by.clone().unwrap_or_default(),
        }
    }
}

impl TemplateHistory {
    pub fn get_property(&self, name: &str) -> String {
        match name {
            "name" => self.name.clone(),
            "body" => self.body.clone(),
            "description" => self.description.clone(),
            "modified_at" => self.modified_at.clone(),
            "user_name" => self.user_name.clone(),
            x => format!("Unhandled property: {x}"),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_action(&self) -> HistoryAction {
        self.change_type.clone()
    }
}
