use crate::database::HistoryAction;
use cloudtruth_restapi::models::TemplateTimelineEntry;

#[derive(Debug, Clone)]
pub struct TemplateHistory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,

    // these are from the timeline
    pub date: String,
    pub change_type: HistoryAction,
    pub user_id: String,
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

            date: api.history_date.clone(),
            change_type: HistoryAction::from(*api.history_type.clone().unwrap_or_default()),
            user_id: api.history_user.clone().unwrap_or_default(),
            user_name: "".to_string(), // must currently be resolved later
        }
    }
}

impl TemplateHistory {
    pub fn get_property(&self, name: &str) -> String {
        match name {
            "name" => self.name.clone(),
            "body" => self.body.clone(),
            "description" => self.description.clone(),
            "user_id" => self.user_id.clone(),
            "user_name" => self.user_name.clone(),
            x => format!("Unhandled property: {x}"),
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
