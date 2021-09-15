use crate::database::openapi::{OpenApiConfig, PAGE_SIZE, WRAP_SECRETS};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::{self, ResponseError};
use cloudtruth_restapi::models::{
    Parameter, ParameterCreate, ParameterRule, ParameterRuleCreate, ParameterRuleTypeEnum,
    ParameterTypeEnum, PatchedParameter, PatchedParameterRule, PatchedValue, Value, ValueCreate,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Formatter};
use std::result::Result;
use std::str::FromStr;

pub struct Parameters {}

static DEFAULT_PARAM_VALUE: OnceCell<Value> = OnceCell::new();
const DEFAULT_VALUE: &str = "-";
const PARTIAL_SUCCESS: Option<bool> = Some(true);
const VALUES_FALSE: Option<bool> = Some(false);
const VALUES_TRUE: Option<bool> = Some(true);

#[derive(Clone, Debug)]
pub struct ParameterDetails {
    // the top few are the parameter, across all environments
    pub id: String,
    pub key: String,
    pub description: String,
    pub secret: bool,
    pub param_type: ParamType,
    pub rules: Vec<ParameterDetailRule>,

    // these come from the value for the specified environment
    pub val_id: String,
    pub value: String,
    pub env_url: String,
    pub env_name: String,
    pub dynamic: bool,
    pub fqn: String,
    pub jmes_path: String,
    pub created_at: String,
    pub modified_at: String,

    // captures errors when fetching dynamic parameters
    pub error: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    String,
    Bool,
    Integer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParamRuleType {
    Max,
    Min,
    MaxLen,
    MinLen,
    Regex,
}

#[derive(Clone, Debug)]
pub struct ParameterDetailRule {
    pub id: String,
    pub rule_type: ParamRuleType,
    pub constraint: String,
    pub created_at: String,
    pub modified_at: String,
}

impl ParameterDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.key.clone(),
            "value" => self.value.clone(),
            "type" => self.param_type.to_string(),
            "environment" => self.env_name.clone(),
            "fqn" => self.fqn.clone(),
            "jmes-path" => self.jmes_path.clone(),
            "description" => self.description.clone(),
            "secret" => format!("{}", self.secret),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
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
        self.env_url = env_value.environment.clone();
        self.dynamic = env_value.dynamic.unwrap_or(false);
        self.fqn = env_value.dynamic_fqn.clone().unwrap_or_default();
        self.jmes_path = env_value.dynamic_filter.clone().unwrap_or_default();
        self.created_at = env_value.created_at.clone();
        self.modified_at = env_value.modified_at.clone();
        self.error = env_value.dynamic_error.clone().unwrap_or_default();
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
            param_type: ParamType::String, // this is the default
            rules: vec![],
            val_id: "".to_string(),
            value: DEFAULT_VALUE.to_string(),
            env_url: "".to_string(),
            env_name: "".to_string(),
            dynamic: false,
            fqn: "".to_string(),
            jmes_path: "".to_string(),
            created_at: "".to_string(),
            modified_at: "".to_string(),
            error: "".to_string(),
        }
    }
}

pub struct ParameterValueEntry {
    pub value: String,
    pub error: String,
}

pub type ParameterDetailMap = HashMap<String, ParameterDetails>;
pub type ParameterValueMap = HashMap<String, ParameterValueEntry>;

/// Gets the singleton default `Value`
fn default_param_value() -> &'static Value {
    DEFAULT_PARAM_VALUE.get_or_init(|| Value {
        url: "".to_owned(),
        id: "".to_owned(),
        environment: "".to_owned(),
        environment_name: "".to_owned(),
        parameter: "".to_owned(),
        dynamic: None,
        dynamic_fqn: None,
        dynamic_filter: None,
        secret: None,
        static_value: None,
        value: Some(DEFAULT_VALUE.to_owned()),
        created_at: "".to_owned(),
        modified_at: "".to_owned(),
        dynamic_error: None,
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
            param_type: ParamType::from(api_param._type.unwrap()),
            rules: api_param
                .rules
                .iter()
                .map(ParameterDetailRule::from)
                .collect(),

            val_id: env_value.id.clone(),
            value: env_value.value.clone().unwrap_or_default(),
            env_url: env_value.environment.clone(),
            env_name: env_value.environment_name.clone(),
            dynamic: env_value.dynamic.unwrap_or(false),
            fqn: env_value.dynamic_fqn.clone().unwrap_or_default(),
            jmes_path: env_value.dynamic_filter.clone().unwrap_or_default(),
            created_at: env_value.created_at.clone(),
            modified_at: env_value.modified_at.clone(),

            error: env_value.dynamic_error.clone().unwrap_or_default(),
        }
    }
}

impl From<ParameterTypeEnum> for ParamType {
    fn from(api: ParameterTypeEnum) -> Self {
        match api {
            ParameterTypeEnum::Bool => Self::Bool,
            ParameterTypeEnum::String => Self::String,
            ParameterTypeEnum::Integer => Self::Integer,
        }
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Integer => write!(f, "integer"),
            Self::String => write!(f, "string"),
        }
    }
}

impl ParamType {
    pub fn to_api_enum(&self) -> ParameterTypeEnum {
        match self {
            Self::String => ParameterTypeEnum::String,
            Self::Bool => ParameterTypeEnum::Bool,
            Self::Integer => ParameterTypeEnum::Integer,
        }
    }
}

impl From<ParameterRuleTypeEnum> for ParamRuleType {
    fn from(api: ParameterRuleTypeEnum) -> Self {
        match api {
            ParameterRuleTypeEnum::Max => Self::Max,
            ParameterRuleTypeEnum::Min => Self::Min,
            ParameterRuleTypeEnum::MaxLen => Self::MaxLen,
            ParameterRuleTypeEnum::MinLen => Self::MinLen,
            ParameterRuleTypeEnum::Regex => Self::Regex,
        }
    }
}

impl From<ParamRuleType> for ParameterRuleTypeEnum {
    fn from(ct: ParamRuleType) -> Self {
        match ct {
            ParamRuleType::Max => Self::Max,
            ParamRuleType::Min => Self::Min,
            ParamRuleType::MaxLen => Self::MaxLen,
            ParamRuleType::MinLen => Self::MinLen,
            ParamRuleType::Regex => Self::Regex,
        }
    }
}

impl fmt::Display for ParamRuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Max => write!(f, "max"),
            Self::Min => write!(f, "min"),
            Self::MaxLen => write!(f, "max-len"),
            Self::MinLen => write!(f, "min-len"),
            Self::Regex => write!(f, "regex"),
        }
    }
}

impl From<&ParameterRule> for ParameterDetailRule {
    fn from(api: &ParameterRule) -> Self {
        Self {
            id: api.id.clone(),
            rule_type: ParamRuleType::from(api._type),
            constraint: api.constraint.clone(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

#[derive(Debug)]
pub enum ParamExportFormat {
    Docker,
    Dotenv,
    Shell,
}

/// Converts to ParamExportFormat from a &str.
impl FromStr for ParamExportFormat {
    type Err = ();

    fn from_str(input: &str) -> Result<ParamExportFormat, Self::Err> {
        match input {
            "docker" => Ok(ParamExportFormat::Docker),
            "dotenv" => Ok(ParamExportFormat::Dotenv),
            "shell" => Ok(ParamExportFormat::Shell),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ParamExportOptions {
    pub format: ParamExportFormat,
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
    pub contains: Option<String>,
    pub export: Option<bool>,
    pub secrets: Option<bool>,
    pub as_of: Option<String>,
}

#[derive(Debug)]
pub enum ParameterError {
    CreateValueError(Error<ProjectsParametersValuesCreateError>),
    UpdateValueError(Error<ProjectsParametersValuesPartialUpdateError>),
    InvalidFqnOrJmesPath(String),
    RuleViolation(String),
    RuleError(String, String),
    UnhandledError(String),
}

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParameterError::InvalidFqnOrJmesPath(msg) => {
                write!(f, "Invalid FQN or JMES path expression: {}", msg)
            }
            ParameterError::RuleViolation(msg) => {
                write!(f, "Rule violation: {}", msg)
            }
            ParameterError::RuleError(action, msg) => {
                write!(f, "Rule {} error: {}", action, msg.replace("_len", "-len"))
            }
            ParameterError::UnhandledError(msg) => {
                write!(f, "Unhandled error: {}", msg)
            }
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for ParameterError {}

fn extract_from_json(value: &serde_json::Value) -> String {
    if value.is_string() {
        return value
            .to_string()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
    }

    if value.is_array() {
        let mut result = "".to_string();
        for v in value.as_array().unwrap() {
            if !result.is_empty() {
                result.push_str("; ")
            }
            // recursively call into this until we have a string
            let obj_str = extract_from_json(v);
            result.push_str(obj_str.as_str());
        }
        return result;
    }

    value.to_string()
}

/// This method is to handle the different errors currently emmited by Value create/update.
fn extract_error(content: &str) -> ParameterError {
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    if let Ok(value) = json_result {
        if let Some(item) = value.get("static_value") {
            return ParameterError::RuleViolation(extract_from_json(item));
        }
        if let Some(item) = value.get("__all__") {
            return ParameterError::InvalidFqnOrJmesPath(extract_from_json(item));
        }
        if let Some(item) = value.get("detail") {
            return ParameterError::UnhandledError(extract_from_json(item));
        }
    }
    ParameterError::UnhandledError(content.to_string())
}

/// Extracts a single string from the content without paying attention to dictionary structure.
fn extract_message(content: &str) -> String {
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    match json_result {
        Ok(value) => extract_from_json(&value),
        _ => "No details available".to_string(),
    }
}

impl Parameters {
    pub fn new() -> Self {
        Self {}
    }

    /// Deletes the specified parameter by ID
    ///
    /// On success, it returns the deleted parameter ID. On failure, it returns an Error.
    pub fn delete_parameter_by_id(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        projects_parameters_destroy(rest_cfg, param_id, proj_id)?;
        Ok(Some(param_id.to_string()))
    }

    /// Deletes the "override" for the specified environment.
    pub fn delete_parameter_value(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersValuesDestroyError>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let response = self.get_details_by_name(rest_cfg, proj_id, env_id, key_name, true, None);

        if let Ok(Some(details)) = response {
            if details.env_url.contains(env_id) {
                projects_parameters_values_destroy(
                    rest_cfg,
                    &details.val_id,
                    &details.id,
                    proj_id,
                )?;
                Ok(Some(details.val_id))
            } else {
                // the "discovered" value is not for this environment
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Exports the specified parameters and values to a well-known output type.
    ///
    /// On success, returns a formatted string containing the specified parameters/values in
    /// the specified output format.
    pub fn export_parameters(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        options: ParamExportOptions,
    ) -> Result<Option<String>, Error<ProjectsParameterExportListError>> {
        let out_fmt = format!("{:?}", options.format).to_lowercase();
        let mask_secrets = Some(!options.secrets.unwrap_or(false));
        let export = projects_parameter_export_list(
            rest_cfg,
            proj_id,
            options.as_of,
            options.contains.as_deref(),
            options.ends_with.as_deref(),
            Some(env_id),
            options.export,
            mask_secrets,
            Some(out_fmt.as_str()),
            options.starts_with.as_deref(),
            WRAP_SECRETS,
        )?;
        Ok(Some(export.body))
    }

    /// Gets the `Parameter` identifier.
    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        key_name: &str,
        as_of: Option<String>,
    ) -> Option<String> {
        // no need to get values/secrets -- just need an ID
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of,
            None,
            Some(true),
            Some(key_name),
            None,
            PAGE_SIZE,
            PARTIAL_SUCCESS,
            VALUES_FALSE,
            WRAP_SECRETS,
        );
        if let Ok(data) = response {
            if let Some(parameters) = data.results {
                if parameters.is_empty() {
                    None
                } else {
                    // TODO: handle more than one?
                    Some(parameters[0].id.clone())
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Fetches the `ParameterDetails` for the specified project/environment/key_name.
    ///
    /// It will return `None` if the parameter does not exist. Other errors will be returned
    /// if project/environments are not found.
    ///
    /// NOTE: the `source` will be the URL, so we don't need another trip to the server to
    ///      change the URL to a name.
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
        mask_secrets: bool,
        as_of: Option<String>,
    ) -> Result<Option<ParameterDetails>, Error<ProjectsParametersListError>> {
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of,
            Some(env_id),
            Some(mask_secrets),
            Some(key_name),
            None,
            PAGE_SIZE,
            PARTIAL_SUCCESS,
            None,
            WRAP_SECRETS,
        )?;
        if let Some(parameters) = response.results {
            if parameters.is_empty() {
                Ok(None)
            } else {
                // TODO: handle more than one??
                let param = &parameters[0];
                Ok(Some(ParameterDetails::from(param)))
            }
        } else {
            Ok(None)
        }
    }

    /// Fetches a "dictionary" of environment variable name/values for the specified project and
    /// environment.
    pub fn get_parameter_values(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        include_values: bool,
        as_of: Option<String>,
    ) -> Result<ParameterValueMap, Error<ProjectsParametersListError>> {
        let parameters = self.get_parameter_details(
            rest_cfg,
            proj_id,
            env_id,
            mask_secrets,
            include_values,
            as_of,
        )?;
        let mut env_vars = ParameterValueMap::new();

        for param in parameters {
            let entry = ParameterValueEntry {
                value: param.value,
                error: param.error,
            };
            env_vars.insert(param.key, entry);
        }
        Ok(env_vars)
    }

    /// Fetches the `ParameterDetails` for the specified project and environment.
    pub fn get_parameter_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        include_values: bool,
        as_of: Option<String>,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list: Vec<ParameterDetails> = Vec::new();
        let env_arg = if include_values { Some(env_id) } else { None };
        let value_arg = if include_values { None } else { VALUES_FALSE };
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of,
            env_arg,
            Some(mask_secrets),
            None,
            None,
            PAGE_SIZE,
            PARTIAL_SUCCESS,
            value_arg,
            WRAP_SECRETS,
        )?;
        if let Some(parameters) = response.results {
            for param in parameters {
                list.push(ParameterDetails::from(&param));
            }
            list.sort_by(|l, r| l.key.cmp(&r.key));
        }
        Ok(list)
    }

    /// Gets a map of parameter names to `ParameterDetails` in the specified environment.
    pub fn get_parameter_detail_map(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        as_of: Option<String>,
    ) -> Result<ParameterDetailMap, Error<ProjectsParametersListError>> {
        let details =
            self.get_parameter_details(rest_cfg, proj_id, env_id, mask_secrets, true, as_of)?;
        let mut result = ParameterDetailMap::new();
        for entry in details {
            result.insert(entry.key.clone(), entry);
        }
        Ok(result)
    }

    /// Gets a map of environment url's to `ParameterDetails` in the specified environment
    pub fn get_parameter_environment_map(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_name: &str,
        mask_secrets: bool,
        as_of: Option<String>,
    ) -> Result<ParameterDetailMap, Error<ProjectsParametersListError>> {
        let mut result = ParameterDetailMap::new();
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of,
            None,
            Some(mask_secrets),
            Some(param_name),
            None,
            PAGE_SIZE,
            PARTIAL_SUCCESS,
            VALUES_TRUE,
            WRAP_SECRETS,
        )?;
        if let Some(values) = response.results {
            for api_param in values {
                let mut details = ParameterDetails::from(&api_param);
                for (_, api_value) in api_param.values {
                    if let Some(value) = api_value {
                        details.set_value(&value);
                        result.insert(details.env_url.clone(), details.clone());
                    }
                }
            }
        }
        Ok(result)
    }

    /// Creates the `Parameter` entry.
    ///
    /// There is no `Value` entry created as part of this -- it is just the `Parameter`.
    pub fn create_parameter(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        key_name: &str,
        description: Option<&str>,
        secret: Option<bool>,
        param_type: Option<ParamType>,
    ) -> Result<ParameterDetails, Error<ProjectsParametersCreateError>> {
        let param_new = ParameterCreate {
            name: key_name.to_string(),
            description: description.map(|x| x.to_string()),
            secret,
            _type: param_type.map(|x| x.to_api_enum()),
        };
        let api_param = projects_parameters_create(rest_cfg, proj_id, param_new)?;
        Ok(ParameterDetails::from(&api_param))
    }

    /// Updates the `Parameter` entry.
    ///
    /// It does not touch any associated `Value` entries.
    #[allow(clippy::too_many_arguments)]
    pub fn update_parameter(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        key_name: &str,
        description: Option<&str>,
        secret: Option<bool>,
        param_type: Option<ParamType>,
    ) -> Result<ParameterDetails, Error<ProjectsParametersPartialUpdateError>> {
        let param_update = PatchedParameter {
            url: None,
            id: None,
            name: Some(key_name.to_string()),
            description: description.map(String::from),
            secret,
            _type: param_type.map(|x| x.to_api_enum()),
            rules: None,
            templates: None,
            values: None,
            created_at: None,
            modified_at: None,
        };
        let api_param =
            projects_parameters_partial_update(rest_cfg, param_id, proj_id, Some(param_update))?;
        Ok(ParameterDetails::from(&api_param))
    }

    /// Creates a `Value` entry associated with the `Parameter` identified by the
    /// `proj_id`/`param_id`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_parameter_value(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        param_id: &str,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<String, ParameterError> {
        let dynamic = value.is_none() || fqn.is_some();
        let value_create = ValueCreate {
            environment: env_id.to_string(),
            dynamic: Some(dynamic),
            static_value: value.map(|v| v.to_string()),
            dynamic_fqn: fqn.map(|v| v.to_string()),
            dynamic_filter: jmes_path.map(|v| v.to_string()),
        };
        let response = projects_parameters_values_create(
            rest_cfg,
            param_id,
            proj_id,
            value_create,
            WRAP_SECRETS,
        );
        match response {
            Ok(api_value) => Ok(api_value.id),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                400 => Err(extract_error(&content.content)),
                404 => Err(extract_error(&content.content)),
                _ => Err(ParameterError::CreateValueError(response.unwrap_err())),
            },
            Err(e) => Err(ParameterError::CreateValueError(e)),
        }
    }

    /// Updates a `Value` entry identified by `proj_id`/`param_id`/`value_id`.
    #[allow(clippy::too_many_arguments)]
    pub fn update_parameter_value(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        value_id: &str,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<String, ParameterError> {
        let dynamic = fqn.is_some() || jmes_path.is_some();
        let value_update = PatchedValue {
            url: None,
            id: None,
            environment: None,
            environment_name: None,
            parameter: None,
            secret: None,
            dynamic: Some(dynamic),
            dynamic_fqn: fqn.map(String::from),
            dynamic_filter: jmes_path.map(String::from),
            static_value: value.map(String::from),
            value: None,
            created_at: None,
            modified_at: None,
            dynamic_error: None,
        };
        let response = projects_parameters_values_partial_update(
            rest_cfg,
            value_id,
            param_id,
            proj_id,
            WRAP_SECRETS,
            Some(value_update),
        );
        match response {
            Ok(api_value) => Ok(api_value.id),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                400 => Err(extract_error(&content.content)),
                404 => Err(extract_error(&content.content)),
                _ => Err(ParameterError::UpdateValueError(response.unwrap_err())),
            },
            Err(e) => Err(ParameterError::UpdateValueError(e)),
        }
    }

    pub fn create_parameter_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        rule_type: ParamRuleType,
        constraint: &str,
    ) -> Result<String, ParameterError> {
        let rule_create = ParameterRuleCreate {
            _type: ParameterRuleTypeEnum::from(rule_type),
            constraint: constraint.to_string(),
        };
        let response = projects_parameters_rules_create(rest_cfg, param_id, proj_id, rule_create);
        let action = "create".to_string();
        match response {
            Ok(rule) => Ok(rule.id),
            Err(ResponseError(ref content)) => Err(ParameterError::RuleError(
                action,
                extract_message(&content.content),
            )),
            Err(e) => Err(ParameterError::RuleError(action, e.to_string())),
        }
    }

    pub fn update_parameter_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        rule_id: &str,
        rule_type: Option<ParamRuleType>,
        constraint: Option<&str>,
    ) -> Result<String, ParameterError> {
        let patch_rule = PatchedParameterRule {
            url: None,
            id: None,
            parameter: None,
            _type: rule_type.map(ParameterRuleTypeEnum::from),
            constraint: constraint.map(String::from),
            created_at: None,
            modified_at: None,
        };
        let response = projects_parameters_rules_partial_update(
            rest_cfg,
            rule_id,
            param_id,
            proj_id,
            Some(patch_rule),
        );
        let action = "update".to_string();
        match response {
            Ok(rule) => Ok(rule.id),
            Err(ResponseError(ref content)) => Err(ParameterError::RuleError(
                action,
                extract_message(&content.content),
            )),
            Err(e) => Err(ParameterError::RuleError(action, e.to_string())),
        }
    }

    pub fn delete_parameter_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        rule_id: &str,
    ) -> Result<String, ParameterError> {
        let response = projects_parameters_rules_destroy(rest_cfg, rule_id, param_id, proj_id);
        let action = "delete".to_string();
        match response {
            Ok(_) => Ok(rule_id.to_string()),
            Err(ResponseError(ref content)) => Err(ParameterError::RuleError(
                action,
                extract_message(&content.content),
            )),
            Err(e) => Err(ParameterError::RuleError(action, e.to_string())),
        }
    }
}
