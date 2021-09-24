use crate::database::openapi::{OpenApiConfig, PAGE_SIZE, WRAP_SECRETS};
use crate::database::{
    extract_from_json, extract_message, generic_response_message, ParamRuleType, ParamType,
    ParameterDetails,
};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::{self, ResponseError};
use cloudtruth_restapi::models::{
    ParameterCreate, ParameterRuleCreate, ParameterRuleTypeEnum, PatchedParameter,
    PatchedParameterRule, PatchedValue, ValueCreate,
};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::result::Result;
use std::str::FromStr;

pub struct Parameters {}

const PARTIAL_SUCCESS: Option<bool> = Some(true);
const VALUES_FALSE: Option<bool> = Some(false);
const VALUES_TRUE: Option<bool> = Some(true);

pub struct ParameterValueEntry {
    pub value: String,
    pub error: String,
}

pub type ParameterDetailMap = HashMap<String, ParameterDetails>;
pub type ParameterValueMap = HashMap<String, ParameterValueEntry>;

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
    pub tag: Option<String>,
}

#[derive(Debug)]
pub enum ParameterError {
    CreateValueError(Error<ProjectsParametersValuesCreateError>),
    UpdateValueError(Error<ProjectsParametersValuesPartialUpdateError>),
    InvalidFqnOrJmesPath(String),
    RuleViolation(String),
    RuleError(String, String),
    UnhandledError(String),
    ResponseError(String),
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
            ParameterError::ResponseError(msg) => {
                write!(f, "{}", msg)
            }
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for ParameterError {}

/// This method is to handle the different errors currently emitted by Value create/update.
fn param_value_error(content: &str) -> ParameterError {
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    if let Ok(value) = json_result {
        if let Some(item) = value.get("internal_value") {
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

/// Creates a `ParameterError::ResponseError` from the provided `status` and `content`
fn param_response_error(status: &reqwest::StatusCode, content: &str) -> ParameterError {
    ParameterError::ResponseError(generic_response_message(status, content))
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
    ) -> Result<Option<String>, ParameterError> {
        let response = projects_parameters_destroy(rest_cfg, param_id, proj_id);
        match response {
            Ok(_) => Ok(Some(param_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(param_response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(format!("{:?}", e))),
        }
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
        let response =
            self.get_details_by_name(rest_cfg, proj_id, env_id, key_name, true, None, None);

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
    ) -> Result<Option<String>, ParameterError> {
        let out_fmt = format!("{:?}", options.format).to_lowercase();
        let mask_secrets = Some(!options.secrets.unwrap_or(false));
        let response = projects_parameter_export_list(
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
            options.tag.as_deref(),
            WRAP_SECRETS,
        );
        match response {
            Ok(export) => Ok(Some(export.body)),
            Err(ResponseError(ref content)) => {
                Err(param_response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(format!("{:?}", e))),
        }
    }

    /// Gets the `Parameter` identifier.
    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        key_name: &str,
    ) -> Option<String> {
        // no need to get values/secrets -- just need an ID (and not tag/time values)
        let as_of_arg = None;
        let env_arg = None;
        let tag_arg = None;
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of_arg,
            env_arg,
            Some(true), // no need to fetch secrets
            Some(key_name),
            None,
            PAGE_SIZE,
            PARTIAL_SUCCESS,
            tag_arg,
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
    #[allow(clippy::too_many_arguments)]
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
        mask_secrets: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Option<ParameterDetails>, ParameterError> {
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
            tag.as_deref(),
            None,
            WRAP_SECRETS,
        );
        match response {
            Ok(data) => match data.results {
                Some(parameters) => {
                    if parameters.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let param = &parameters[0];
                        Ok(Some(ParameterDetails::from(param)))
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(param_response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(format!("{:?}", e))),
        }
    }

    /// Fetches a "dictionary" of environment variable name/values for the specified project and
    /// environment.
    #[allow(clippy::too_many_arguments)]
    pub fn get_parameter_values(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        include_values: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<ParameterValueMap, ParameterError> {
        let parameters = self.get_parameter_details(
            rest_cfg,
            proj_id,
            env_id,
            mask_secrets,
            include_values,
            as_of,
            tag,
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
    #[allow(clippy::too_many_arguments)]
    pub fn get_parameter_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        include_values: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Vec<ParameterDetails>, ParameterError> {
        let has_values = include_values || tag.is_some();
        let env_arg = if has_values { Some(env_id) } else { None };
        let value_arg = if has_values { None } else { VALUES_FALSE };
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
            tag.as_deref(),
            value_arg,
            WRAP_SECRETS,
        );
        match response {
            Ok(data) => {
                let mut list: Vec<ParameterDetails> = Vec::new();
                if let Some(parameters) = data.results {
                    for param in parameters {
                        list.push(ParameterDetails::from(&param));
                    }
                    list.sort_by(|l, r| l.key.cmp(&r.key));
                }
                Ok(list)
            }
            Err(ResponseError(ref content)) => {
                Err(param_response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(format!("{:?}", e))),
        }
    }

    /// Gets a map of parameter names to `ParameterDetails` in the specified environment.
    pub fn get_parameter_detail_map(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<ParameterDetailMap, ParameterError> {
        let details =
            self.get_parameter_details(rest_cfg, proj_id, env_id, mask_secrets, true, as_of, tag)?;
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
        tag: Option<String>,
    ) -> Result<ParameterDetailMap, ParameterError> {
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
            tag.as_deref(),
            VALUES_TRUE,
            WRAP_SECRETS,
        );
        match response {
            Ok(data) => {
                let mut result = ParameterDetailMap::new();
                if let Some(values) = data.results {
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
            Err(ResponseError(ref content)) => {
                Err(param_response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(format!("{:?}", e))),
        }
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
        let external = value.is_none() || fqn.is_some();
        let value_create = ValueCreate {
            environment: env_id.to_string(),
            external: Some(external),
            internal_value: value.map(|v| v.to_string()),
            external_fqn: fqn.map(|v| v.to_string()),
            external_filter: jmes_path.map(|v| v.to_string()),
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
                400 => Err(param_value_error(&content.content)),
                404 => Err(param_value_error(&content.content)),
                _ => Err(param_response_error(&content.status, &content.content)),
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
        let external = fqn.is_some() || jmes_path.is_some();
        let value_update = PatchedValue {
            url: None,
            id: None,
            environment: None,
            environment_name: None,
            parameter: None,
            secret: None,
            external: Some(external),
            external_fqn: fqn.map(String::from),
            external_filter: jmes_path.map(String::from),
            internal_value: value.map(String::from),
            value: None,
            created_at: None,
            modified_at: None,
            external_error: None,
            earliest_tag: None,
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
                400 => Err(param_value_error(&content.content)),
                404 => Err(param_value_error(&content.content)),
                _ => Err(param_response_error(&content.status, &content.content)),
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
