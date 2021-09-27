use crate::database::HistoryAction;
use cloudtruth_restapi::models::TemplateTimelineEntry;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct TemplateHistory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,

    // these are from the timeline
    pub date: String,
    pub change_type: HistoryAction,
    pub user: String, // may need name and the id
}

impl From<&TemplateTimelineEntry> for TemplateHistory {
    fn from(api: &TemplateTimelineEntry) -> Self {
        TemplateHistory {
            id: api.history_template.id.clone(),
            name: api.history_template.name.clone(),
            description: api.history_template.description.clone().unwrap_or_default(),
            body: api.history_template.body.clone().unwrap_or_default(),

            date: api.history_date.clone(),
            change_type: HistoryAction::from(*api.history_type.deref()),
            user: api.history_user.clone().unwrap_or_default(),
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

    pub fn get_date(&self) -> String {
        self.date.clone()
    }

    pub fn get_action(&self) -> HistoryAction {
        self.change_type.clone()
    }
}
