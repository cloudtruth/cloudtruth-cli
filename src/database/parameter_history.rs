use crate::database::HistoryAction;
use cloudtruth_restapi::models::{
    ParameterTimelineEntry, ParameterTimelineEntryEnvironment, ParameterTimelineEntryParameter,
};
use once_cell::sync::OnceCell;

static DEFAULT_ENV_HISTORY: OnceCell<ParameterTimelineEntryEnvironment> = OnceCell::new();
static DEFAULT_PARAM_HISTORY: OnceCell<ParameterTimelineEntryParameter> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct ParameterHistory {
    pub id: String,
    pub name: String,

    // TODO: can we get description, value, rules, FQN, jmes_path??
    pub env_name: String,

    // these are from the timeline
    pub date: String,
    pub change_type: HistoryAction,
    pub user: String,
}

/// Gets the singleton default History
fn default_environment_history() -> &'static ParameterTimelineEntryEnvironment {
    DEFAULT_ENV_HISTORY.get_or_init(ParameterTimelineEntryEnvironment::default)
}

/// Gets the singleton default History
fn default_parameter_history() -> &'static ParameterTimelineEntryParameter {
    DEFAULT_PARAM_HISTORY.get_or_init(ParameterTimelineEntryParameter::default)
}

impl From<&ParameterTimelineEntry> for ParameterHistory {
    fn from(api: &ParameterTimelineEntry) -> Self {
        let first = api.history_environments.first();
        let env_hist: &ParameterTimelineEntryEnvironment = match first {
            Some(v) => v,
            _ => default_environment_history(),
        };
        let param_hist = match &api.history_parameter {
            Some(p) => &*p,
            _ => default_parameter_history(),
        };

        Self {
            id: param_hist.id.clone(),
            name: param_hist.name.clone(),

            env_name: env_hist.name.clone(),

            date: api.history_date.clone(),
            change_type: HistoryAction::from(*api.history_type.clone().unwrap_or_default()),
            user: api.history_user.clone().unwrap_or_default(),
        }
    }
}

impl ParameterHistory {
    pub fn get_property(&self, name: &str) -> String {
        match name {
            "name" => self.name.clone(),
            "environment" => self.env_name.clone(),
            // TODO: add more here once available
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
