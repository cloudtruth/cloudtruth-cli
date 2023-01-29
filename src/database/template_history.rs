use crate::database::HistoryAction;
use cloudtruth_restapi::models::TemplateTimelineEntry;

#[derive(Debug, Clone)]
pub struct TemplateHistory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub change_type: HistoryAction,
}

impl From<&TemplateTimelineEntry> for TemplateHistory {
    fn from(api: &TemplateTimelineEntry) -> Self {
        let api_template = api.history_template.clone().unwrap_or_default();
        TemplateHistory {
            id: api_template.id.clone(),
            name: api_template.name.clone(),
            description: api_template.description.clone().unwrap_or_default(),
            body: api_template.body.clone().unwrap_or_default(),
            change_type: HistoryAction::from(*api.history_type.clone().unwrap_or_default())
        }
    }
}

impl TemplateHistory {
    pub fn get_property(&self, name: &str) -> String {
        match name {
            "name" => self.name.clone(),
            "body" => self.body.clone(),
            "description" => self.description.clone(),
            x => format!("Unhandled property: {}", x),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_action(&self) -> HistoryAction {
        self.change_type.clone()
    }
}
