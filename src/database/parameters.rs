use crate::database::openapi::key_from_config;
use crate::database::{
    extract_details, extract_from_json, page_size, response_message, secret_encode_wrap,
    secret_unwrap_decode, CryptoAlgorithm, OpenApiConfig, ParamExportOptions, ParamRuleType,
    ParameterDetails, ParameterError, TaskStepDetails, NO_PAGE_COUNT, NO_PAGE_SIZE, WRAP_SECRETS,
};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::utils_api::utils_generate_password_create;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    ParameterCreate, ParameterRuleCreate, ParameterRuleTypeEnum, PatchedParameter,
    PatchedParameterRule, PatchedValue, ValueCreate,
};
use std::collections::HashMap;
use std::result::Result;

const VALUES_FALSE: Option<bool> = Some(false);
const VALUES_TRUE: Option<bool> = Some(true);
const NO_ORDERING: Option<&str> = None;
const ONLY_SECRETS: Option<bool> = None;
const NO_DESC_ICONTAINS: Option<&str> = None;
const NO_ID_IN: Option<Vec<String>> = None;
const NO_NAME_CONTAINS: Option<&str> = None;
const NO_NAME_ICONTAINS: Option<&str> = None;
const NO_NAME_IEXACT: Option<&str> = None;
const NO_NAME_ISTARTS: Option<&str> = None;
const NO_NAME_STARTS: Option<&str> = None;

const WRAP_ALGORITHM: CryptoAlgorithm = CryptoAlgorithm::AesGcm;

pub struct Parameters {}

pub struct ParameterValueEntry {
    pub value: String,
    pub error: String,
}

pub type ParameterDetailMap = HashMap<String, ParameterDetails>;
pub type ParameterValueMap = HashMap<String, ParameterValueEntry>;

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
        // template evaluation failures seems to produce an array of strings...
        if value.is_array() {
            return ParameterError::EvaluationError(extract_from_json(&value));
        }
    }
    ParameterError::UnhandledError(content.to_string())
}

/// Creates a `ParameterError::ResponseError` from the provided `status` and `content`
fn response_error(status: &reqwest::StatusCode, content: &str) -> ParameterError {
    let prefix = "\n  ";
    match status.as_u16() {
        422 => ParameterError::EvaluationError(format!(
            "{}{}",
            prefix,
            extract_details(content).replace("; ", prefix)
        )),
        _ => ParameterError::ResponseError(response_message(status, content)),
    }
}

fn rule_error(action: String, content: &str) -> ParameterError {
    ParameterError::RuleError(action, extract_details(content))
}

fn mask_secrets_arg(mask_secrets: bool) -> Option<bool> {
    match mask_secrets {
        true => Some(true),
        false => None,
    }
}

fn wrap_secrets_arg(mask_secrets: bool) -> Option<bool> {
    match mask_secrets {
        true => None,
        false => Some(WRAP_SECRETS),
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
    ) -> Result<Option<String>, ParameterError> {
        let response = projects_parameters_destroy(rest_cfg, param_id, proj_id);
        match response {
            Ok(_) => Ok(Some(param_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
        }
    }

    /// Deletes the "override" for the specified environment.
    pub fn delete_parameter_value(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<String>, ParameterError> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let evaluate = false; // no need to evaluate things
        let mask_secrets = true; // no need to fetch secrets
        let response = self.get_details_by_name(
            rest_cfg,
            proj_id,
            env_id,
            key_name,
            evaluate,
            mask_secrets,
            None,
            None,
        );

        if let Ok(Some(details)) = response {
            if details.env_url.contains(env_id) {
                let del_resp = projects_parameters_values_destroy(
                    rest_cfg,
                    &details.val_id,
                    &details.id,
                    proj_id,
                    None,
                );
                match del_resp {
                    Ok(_) => Ok(Some(details.val_id)),
                    Err(ResponseError(ref content)) => {
                        Err(response_error(&content.status, &content.content))
                    }
                    Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
                }
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
        let mask_secrets = !options.secrets.unwrap_or(false);
        let response = projects_parameter_export_list(
            rest_cfg,
            proj_id,
            options.as_of,
            options.contains.as_deref(),
            options.ends_with.as_deref(),
            Some(env_id),
            options.export,
            mask_secrets_arg(mask_secrets),
            NO_ORDERING,
            Some(out_fmt.as_str()),
            options.starts_with.as_deref(),
            options.tag.as_deref(),
            None, // TODO: should wrap per wrap_secrets_arg(), but makes output text unusable
        );
        match response {
            Ok(export) => Ok(Some(export.body)),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsParameterExportListError::Status422(tle)) => {
                    Err(ParameterError::TemplateEvalError(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
        evaluate: bool,
        mask_secrets: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Option<ParameterDetails>, ParameterError> {
        let env_arg = if !env_id.is_empty() {
            Some(env_id)
        } else {
            None
        };
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            as_of,
            NO_DESC_ICONTAINS,
            None,
            env_arg,
            Some(evaluate),
            NO_ID_IN,
            mask_secrets_arg(mask_secrets),
            Some(key_name),
            NO_NAME_CONTAINS,
            NO_NAME_ICONTAINS,
            NO_NAME_IEXACT,
            NO_NAME_ISTARTS,
            NO_NAME_STARTS,
            NO_ORDERING,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
            ONLY_SECRETS,
            tag.as_deref(),
            None,
            wrap_secrets_arg(mask_secrets),
        );
        match response {
            Ok(data) => match data.results {
                Some(parameters) => {
                    if parameters.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let param = &parameters[0];
                        let mut details = ParameterDetails::from(param);
                        if WRAP_SECRETS && !mask_secrets && details.encrypted() {
                            let key = key_from_config(rest_cfg);
                            let plaintext = secret_unwrap_decode(key.as_bytes(), &details.value)?;
                            details.value = plaintext;
                        }
                        Ok(Some(details))
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
        let mut result: Vec<ParameterDetails> = Vec::new();
        let mut page_count = 1;
        loop {
            let has_values = include_values || tag.is_some();
            let env_arg = if has_values { Some(env_id) } else { None };
            let value_arg = if has_values { None } else { VALUES_FALSE };
            let eval_arg = Some(include_values);
            let response = projects_parameters_list(
                rest_cfg,
                proj_id,
                as_of.clone(),
                NO_DESC_ICONTAINS,
                None,
                env_arg,
                eval_arg,
                NO_ID_IN,
                mask_secrets_arg(mask_secrets),
                None,
                NO_NAME_CONTAINS,
                NO_NAME_ICONTAINS,
                NO_NAME_IEXACT,
                NO_NAME_ISTARTS,
                NO_NAME_STARTS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                ONLY_SECRETS,
                tag.as_deref(),
                value_arg,
                wrap_secrets_arg(mask_secrets),
            );
            match response {
                Ok(data) => {
                    if let Some(parameters) = data.results {
                        for param in parameters {
                            let mut details = ParameterDetails::from(&param);
                            if WRAP_SECRETS && !mask_secrets && details.encrypted() {
                                let key = key_from_config(rest_cfg);
                                let plaintext =
                                    secret_unwrap_decode(key.as_bytes(), &details.value)?;
                                details.value = plaintext;
                            }
                            result.push(details);
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(ParameterError::UnhandledError(e.to_string())),
            }
        }
        result.sort_by(|l, r| l.key.cmp(&r.key));
        Ok(result)
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
    ) -> Result<ParameterDetailMap, ParameterError> {
        let mut result = ParameterDetailMap::new();
        let mut page_count = 1;
        loop {
            let eval_arg = VALUES_TRUE;
            let response = projects_parameters_list(
                rest_cfg,
                proj_id,
                as_of.clone(),
                NO_DESC_ICONTAINS,
                None,
                None, // cannot give an environment, or it will only get for that environment
                eval_arg,
                NO_ID_IN,
                mask_secrets_arg(mask_secrets),
                Some(param_name),
                NO_NAME_CONTAINS,
                NO_NAME_ICONTAINS,
                NO_NAME_IEXACT,
                NO_NAME_ISTARTS,
                NO_NAME_STARTS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                ONLY_SECRETS,
                None, // cannot use a tag without an environment
                VALUES_TRUE,
                wrap_secrets_arg(mask_secrets),
            );
            match response {
                Ok(data) => {
                    let key = key_from_config(rest_cfg);
                    if let Some(values) = data.results {
                        for api_param in values {
                            let mut details = ParameterDetails::from(&api_param);
                            for (_, api_value) in api_param.values {
                                if let Some(value) = api_value {
                                    details.set_value(&value);
                                    if WRAP_SECRETS && !mask_secrets && details.encrypted() {
                                        details.value =
                                            secret_unwrap_decode(key.as_bytes(), &details.value)?;
                                    }
                                    result.insert(details.env_url.clone(), details.clone());
                                }
                            }
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(ParameterError::UnhandledError(e.to_string())),
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
        param_type: Option<&str>,
    ) -> Result<ParameterDetails, ParameterError> {
        let param_new = ParameterCreate {
            name: key_name.to_string(),
            description: description.map(String::from),
            secret,
            _type: param_type.map(String::from),
        };
        let response = projects_parameters_create(rest_cfg, proj_id, param_new);
        match response {
            Ok(api_param) => Ok(ParameterDetails::from(&api_param)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
        }
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
        param_type: Option<&str>,
    ) -> Result<ParameterDetails, ParameterError> {
        let param_update = PatchedParameter {
            url: None,
            id: None,
            name: Some(key_name.to_string()),
            description: description.map(String::from),
            secret,
            _type: param_type.map(String::from),
            rules: None,
            values: None,
            referencing_templates: None,
            referencing_values: None,
            created_at: None,
            modified_at: None,
            project: None,
            project_name: None,
            overrides: None,
        };
        let response =
            projects_parameters_partial_update(rest_cfg, param_id, proj_id, Some(param_update));
        match response {
            Ok(api_param) => Ok(ParameterDetails::from(&api_param)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
        }
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
        is_secret: bool,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
        evaluated: Option<bool>,
    ) -> Result<String, ParameterError> {
        let external = value.is_none() || fqn.is_some();
        let data = match value {
            Some(v) => {
                if is_secret && WRAP_SECRETS {
                    let key = key_from_config(rest_cfg);
                    let encrypted = secret_encode_wrap(WRAP_ALGORITHM, key.as_bytes(), v).unwrap();
                    Some(encrypted)
                } else {
                    Some(String::from(v))
                }
            }
            None => None,
        };
        let value_create = ValueCreate {
            environment: env_id.to_string(),
            external: Some(external),
            internal_value: data,
            external_fqn: fqn.map(|v| v.to_string()),
            external_filter: jmes_path.map(|v| v.to_string()),
            interpolated: evaluated,
        };
        let response = projects_parameters_values_create(
            rest_cfg,
            param_id,
            proj_id,
            value_create,
            None,
            Some(WRAP_SECRETS),
        );
        match response {
            Ok(api_value) => Ok(api_value.id),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                400 => Err(param_value_error(&content.content)),
                404 => Err(param_value_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
        is_secret: bool,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
        evaluated: Option<bool>,
    ) -> Result<String, ParameterError> {
        let external = fqn.is_some() || jmes_path.is_some();
        let data = match value {
            Some(v) => {
                if is_secret && WRAP_SECRETS {
                    let key = key_from_config(rest_cfg);
                    let encrypted = secret_encode_wrap(WRAP_ALGORITHM, key.as_bytes(), v).unwrap();
                    Some(encrypted)
                } else {
                    Some(String::from(v))
                }
            }
            None => None,
        };
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
            internal_value: data,
            value: None,
            created_at: None,
            modified_at: None,
            external_error: None,
            earliest_tag: None,
            interpolated: evaluated,
            evaluated: None,
            external_status: None,
            referenced_parameters: None,
            referenced_templates: None,
        };
        let response = projects_parameters_values_partial_update(
            rest_cfg,
            value_id,
            param_id,
            proj_id,
            None,
            Some(WRAP_SECRETS),
            Some(value_update),
        );
        match response {
            Ok(api_value) => Ok(api_value.id),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                400 => Err(param_value_error(&content.content)),
                404 => Err(param_value_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
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
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_task_steps(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        param_id: &str,
    ) -> Result<Vec<TaskStepDetails>, ParameterError> {
        let mut result = vec![];
        let mut page_count = 1;
        loop {
            let response = projects_parameters_pushes_list(
                rest_cfg,
                param_id,
                proj_id,
                None,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(list) = data.results {
                        for ref task in list {
                            if task.environment_id.clone().unwrap_or_default().as_str() == env_id {
                                result.push(TaskStepDetails::from(task));
                            }
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(ParameterError::UnhandledError(e.to_string())),
            }
        }
        Ok(result)
    }

    pub fn get_all_task_steps(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
    ) -> Result<Vec<TaskStepDetails>, ParameterError> {
        // need the parameter id for getting task steps, so get list of parameters
        let params = self.get_parameter_details(rest_cfg, proj_id, "", true, false, None, None)?;
        let mut total = vec![];
        for p in params {
            let mut tasks = self.get_task_steps(rest_cfg, proj_id, env_id, &p.id)?;
            total.append(&mut tasks);
        }
        Ok(total)
    }

    /// Use the API to generate a new password according to the provided policy flags
    #[allow(clippy::too_many_arguments)]
    pub fn generate_password(
        &self,
        rest_cfg: &OpenApiConfig,
        length: i32,
        require_hardware_generation: Option<bool>,
        require_lowercase: Option<bool>,
        require_numbers: Option<bool>,
        require_spaces: Option<bool>,
        require_symbols: Option<bool>,
        require_uppercase: Option<bool>,
    ) -> Result<String, ParameterError> {
        let response = utils_generate_password_create(
            rest_cfg,
            length,
            require_hardware_generation,
            require_lowercase,
            require_numbers,
            require_spaces,
            require_symbols,
            require_uppercase,
        );
        match response {
            Ok(data) => Ok(data.value),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ParameterError::UnhandledError(e.to_string())),
        }
    }
}
