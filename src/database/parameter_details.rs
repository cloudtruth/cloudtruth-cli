use crate::database::{ParamRuleType, ParameterRuleDetail};
use cloudtruth_restapi::models::{Parameter, Value};
use once_cell::sync::OnceCell;

static DEFAULT_PARAM_VALUE: OnceCell<Value> = OnceCell::new();
const DEFAULT_VALUE: &str = "-";

#[derive(Clone, Debug)]
pub struct ParameterDetails {
    // the top few are the parameter, across all environments
    pub id: String,
    pub key: String,
    pub description: String,
    pub secret: bool,
    pub param_type: String,
    pub rules: Vec<ParameterRuleDetail>,
    pub project_url: String,
    pub project_name: String,

    // these come from the value for the specified environment
    pub val_id: String,
    pub value: String,
    pub val_url: String,
    pub env_url: String,
    pub env_name: String,
    pub external: bool,
    pub fqn: String,
    pub jmes_path: String,
    pub evaluated: bool,
    pub raw_value: String, // the unevaluated value
    pub created_at: String,
    pub modified_at: String,

    // captures errors when fetching external parameters
    pub error: String,
}

impl ParameterDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "name" => self.key.clone(),
            "value" => self.value.clone(),
            "type" => self.param_type.clone(),
            "environment" => self.env_name.clone(),
            "fqn" => self.fqn.clone(),
            "jmes-path" => self.jmes_path.clone(),
            "description" => self.description.clone(),
            "secret" => format!("{}", self.secret),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            "rule-count" => self.rules.len().to_string(),
            "raw" => self.raw_value.clone(),
            "scope" => {
                let mut scope = String::from(if self.external {
                    "external"
                } else {
                    "internal"
                });
                if self.evaluated {
                    scope.push_str("-evaluated");
                }
                scope
            }
            "project-url" => self.project_url.clone(),
            "project-name" => self.project_name.clone(),
            _ => format!("Unhandled property name '{property_name}'"),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }

    /// Updates the values associated with an API `Value`.
    ///
    /// This is used for iteration over a list of values.
    pub fn set_value(&mut self, env_value: &Value) {
        self.val_id = env_value.id.clone();
        self.value = env_value.value.clone().unwrap_or_default();
        self.val_url = env_value.url.clone();
        self.env_url = env_value.environment.replace("http://", "https://");
        self.external = env_value.external.unwrap_or(false);
        self.fqn = env_value.external_fqn.clone().unwrap_or_default();
        self.jmes_path = env_value.external_filter.clone().unwrap_or_default();
        self.evaluated = env_value.interpolated.unwrap_or(false);
        self.raw_value = env_value.internal_value.clone().unwrap_or_default();
        self.created_at = env_value.created_at.clone();
        self.modified_at = env_value.modified_at.clone().unwrap_or_default();
        self.error = env_value.external_error.clone().unwrap_or_default();
    }

    /// Gets the first id matching the provided type
    pub fn get_rule_id(&self, rule_type: ParamRuleType) -> Option<String> {
        let mut result: Option<String> = None;
        for rule in &self.rules {
            if rule.rule_type == rule_type {
                result = Some(rule.id.clone());
                break;
            }
        }
        result
    }
}

impl Default for ParameterDetails {
    fn default() -> Self {
        ParameterDetails {
            id: "".to_string(),
            key: "".to_string(),
            description: "".to_string(),
            secret: false,
            param_type: "string".to_string(),
            rules: vec![],
            project_url: "".to_string(),
            project_name: "".to_string(),
            val_id: "".to_string(),
            value: DEFAULT_VALUE.to_string(),
            val_url: "".to_string(),
            env_url: "".to_string(),
            env_name: "".to_string(),
            external: false,
            fqn: "".to_string(),
            jmes_path: "".to_string(),
            evaluated: false,
            raw_value: "".to_string(),
            created_at: "".to_string(),
            modified_at: "".to_string(),
            error: "".to_string(),
        }
    }
}

/// Gets the singleton default `Value`
fn default_param_value() -> &'static Value {
    DEFAULT_PARAM_VALUE.get_or_init(|| Value {
        url: "".to_owned(),
        id: "".to_owned(),
        ledger_id: "".to_owned(),
        environment: "".to_owned(),
        environment_name: "".to_owned(),
        environment_id: "".to_owned(),
        active_environment: "".to_owned(),
        active_environment_id: "".to_owned(),
        active_environment_name: "".to_owned(),
        parameter: "".to_owned(),
        parameter_id: "".to_owned(),
        external: None,
        external_fqn: None,
        external_filter: None,
        external_status: None,
        secret: None,
        internal_value: None,
        interpolated: None,
        evaluated: false,
        referenced_projects: vec![],
        referenced_parameters: vec![],
        referenced_templates: vec![],
        value: Some(DEFAULT_VALUE.to_owned()),
        created_at: "".to_owned(),
        modified_at: None,
        external_error: None,
    })
}

impl From<&Parameter> for ParameterDetails {
    fn from(api_param: &Parameter) -> Self {
        let first = api_param.values.values().next();
        let env_value: &Value = match first {
            Some(Some(v)) => v,
            _ => default_param_value(),
        };

        ParameterDetails {
            id: api_param.id.clone(),
            key: api_param.name.clone(),
            secret: api_param.secret.unwrap_or(false) || env_value.secret.unwrap_or(false),
            description: api_param.description.clone().unwrap_or_default(),
            param_type: api_param._type.clone().unwrap_or_default(),
            project_url: api_param.project.clone(),
            project_name: api_param.project_name.clone(),
            rules: api_param
                .rules
                .iter()
                .map(ParameterRuleDetail::from)
                .collect(),

            val_id: env_value.id.clone(),
            value: env_value.value.clone().unwrap_or_default(),
            val_url: env_value.url.clone(),
            env_url: env_value.environment.clone(),
            env_name: env_value.environment_name.clone(),
            external: env_value.external.unwrap_or(false),
            fqn: env_value.external_fqn.clone().unwrap_or_default(),
            jmes_path: env_value.external_filter.clone().unwrap_or_default(),
            evaluated: env_value.interpolated.unwrap_or(false),
            raw_value: env_value.internal_value.clone().unwrap_or_default(),
            created_at: env_value.created_at.clone(),
            modified_at: env_value.modified_at.clone().unwrap_or_default(),

            error: env_value.external_error.clone().unwrap_or_default(),
        }
    }
}
