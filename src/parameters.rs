use crate::environments::{EnvironmentUrlMap, Environments};
use crate::openapi::{extract_details, OpenApiConfig, PAGE_SIZE, WRAP_SECRETS};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::{self, ResponseError};
use cloudtruth_restapi::models::{
    Parameter, ParameterCreate, PatchedParameter, PatchedValue, Value, ValueCreate,
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

#[derive(Debug)]
pub struct ParameterDetails {
    // the top few are the parameter, across all environments
    pub id: String,
    pub key: String,
    pub description: String,
    pub secret: bool,

    // these come from the value for the specified environment
    pub val_id: String,
    pub value: String,
    pub env_url: String,
    pub env_name: String,
    pub dynamic: bool,
    pub fqn: String,
    pub jmes_path: String,

    // captures errors when fetching dynamic parameters
    pub error: String,
}

impl ParameterDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.key.clone(),
            "value" => self.value.clone(),
            "environment" => self.env_name.clone(),
            "fqn" => self.fqn.clone(),
            "jmes-path" => self.jmes_path.clone(),
            "description" => self.description.clone(),
            "secret" => format!("{}", self.secret),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl Default for ParameterDetails {
    fn default() -> Self {
        ParameterDetails {
            id: "".to_string(),
            key: "".to_string(),
            description: "".to_string(),
            secret: false,
            val_id: "".to_string(),
            value: DEFAULT_VALUE.to_string(),
            env_url: "".to_string(),
            env_name: "".to_string(),
            dynamic: false,
            fqn: "".to_string(),
            jmes_path: "".to_string(),
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
        let env_value: &Value = match first.unwrap() {
            Some(opt) => opt,
            None => default_param_value(),
        };

        ParameterDetails {
            id: api_param.id.clone(),
            key: api_param.name.clone(),
            secret: api_param.secret.unwrap_or(false) || env_value.secret.unwrap_or(false),
            description: api_param.description.clone().unwrap_or_default(),

            val_id: env_value.id.clone(),
            value: env_value.value.clone().unwrap_or_default(),
            env_url: env_value.environment.clone(),
            env_name: "".to_owned(),
            dynamic: env_value.dynamic.unwrap_or(false),
            fqn: env_value.dynamic_fqn.clone().unwrap_or_default(),
            jmes_path: env_value.dynamic_filter.clone().unwrap_or_default(),

            error: env_value.dynamic_error.clone().unwrap_or_default(),
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
}

#[derive(Debug)]
pub enum ParameterValueError {
    CreateError(Error<ProjectsParametersValuesCreateError>),
    UpdateError(Error<ProjectsParametersValuesPartialUpdateError>),
    InvalidFqnOrJmesPath(String),
    FqnOrJmesPathNotFound(String),
}

impl fmt::Display for ParameterValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParameterValueError::InvalidFqnOrJmesPath(msg) => {
                write!(f, "Invalid FQN or JMES path expression: {}", msg)
            }
            ParameterValueError::FqnOrJmesPathNotFound(msg) => {
                write!(f, "Did not find FQN or JMES path: {}", msg)
            }
            ParameterValueError::CreateError(e) => {
                write!(f, "{}", e.to_string())
            }
            ParameterValueError::UpdateError(e) => {
                write!(f, "{}", e.to_string())
            }
        }
    }
}

impl error::Error for ParameterValueError {}

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

    /// Deletes the specified parameter from the specified project/environment.
    ///
    /// On success, it returns the ID of the deleted value. On failure, it returns an Error
    /// with more failure information.
    pub fn delete_parameter(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let response = self.get_id(rest_cfg, proj_id, env_id, key_name);

        if let Some(id) = response {
            self.delete_parameter_by_id(rest_cfg, proj_id, &id)
        } else {
            Ok(None)
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
        let response = self.get_details_by_name(rest_cfg, proj_id, env_id, key_name, true);

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
            options.contains.as_deref(),
            options.ends_with.as_deref(),
            Some(env_id),
            options.export,
            mask_secrets,
            Some(out_fmt.as_str()),
            None,
            PAGE_SIZE,
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
        env_id: &str,
        key_name: &str,
    ) -> Option<String> {
        // NOTE: should say "No Values" when that's an option
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            Some(env_id),
            Some(true),
            Some(key_name),
            None,
            PAGE_SIZE,
            Some(true),
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
    ) -> Result<Option<ParameterDetails>, Error<ProjectsParametersRetrieveError>> {
        if let Some(id) = self.get_id(rest_cfg, proj_id, env_id, key_name) {
            let response = projects_parameters_retrieve(
                rest_cfg,
                &id,
                proj_id,
                Some(env_id),
                Some(mask_secrets),
                Some(true),
                WRAP_SECRETS,
            )?;
            Ok(Some(ParameterDetails::from(&response)))
        } else {
            Ok(None)
        }
    }

    /// This is the original `get_details_by_name()` where the parameter/value are resolved in one
    /// query. However, this currently causes performance issues that are being worked on.
    #[allow(dead_code)]
    pub fn _get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
        mask_secrets: bool,
    ) -> Result<Option<ParameterDetails>, Error<ProjectsParametersListError>> {
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            Some(env_id),
            Some(mask_secrets),
            Some(key_name),
            None,
            PAGE_SIZE,
            Some(true),
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
    ) -> Result<ParameterValueMap, Error<ProjectsParametersListError>> {
        let parameters =
            self.get_parameter_unresolved_details(rest_cfg, proj_id, env_id, mask_secrets)?;
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
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list =
            self.get_parameter_unresolved_details(rest_cfg, proj_id, env_id, mask_secrets)?;

        // now, resolve the source URL to the source environment name
        let environments = Environments::new();
        let url_map = environments.get_url_name_map(rest_cfg);
        self.resolve_environments(&url_map, &mut list);
        Ok(list)
    }

    /// Resolves the `env_name` field in the `ParameterDetails` object by interrogating the
    /// `EnvironmentUrlMap` with the `env_url` field.
    fn resolve_environments(
        &self,
        env_url_map: &EnvironmentUrlMap,
        list: &mut Vec<ParameterDetails>,
    ) {
        let default_key = "".to_string();
        for details in list {
            details.env_name = env_url_map
                .get(&details.env_url)
                .unwrap_or(&default_key)
                .clone();
        }
    }

    /// This internal function gets the `ParameterDetails`, but does not resolve the `source` from
    /// the URL to the name.
    fn get_parameter_unresolved_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list: Vec<ParameterDetails> = Vec::new();
        let response = projects_parameters_list(
            rest_cfg,
            proj_id,
            Some(env_id),
            Some(mask_secrets),
            None,
            None,
            PAGE_SIZE,
            Some(true),
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

    pub fn get_parameter_detail_map(
        &self,
        rest_cfg: &OpenApiConfig,
        env_url_map: &EnvironmentUrlMap,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
    ) -> Result<ParameterDetailMap, Error<ProjectsParametersListError>> {
        let mut details =
            self.get_parameter_unresolved_details(rest_cfg, proj_id, env_id, mask_secrets)?;
        self.resolve_environments(env_url_map, &mut details);
        let mut result = ParameterDetailMap::new();
        for entry in details {
            result.insert(entry.key.clone(), entry);
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
    ) -> Result<ParameterDetails, Error<ProjectsParametersCreateError>> {
        let param_new = ParameterCreate {
            name: key_name.to_string(),
            description: description.map(|x| x.to_string()),
            secret,
        };
        let api_param = projects_parameters_create(rest_cfg, proj_id, param_new)?;
        Ok(ParameterDetails::from(&api_param))
    }

    /// Updates the `Parameter` entry.
    ///
    /// It does not touch any associated `Value` entries.
    pub fn update_parameter(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        param_id: &str,
        key_name: &str,
        description: Option<&str>,
        secret: Option<bool>,
    ) -> Result<ParameterDetails, Error<ProjectsParametersPartialUpdateError>> {
        let param_update = PatchedParameter {
            url: None,
            id: None,
            name: Some(key_name.to_string()),
            description: description.map(String::from),
            secret,
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
    ) -> Result<String, ParameterValueError> {
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
                400 => Err(ParameterValueError::InvalidFqnOrJmesPath(extract_details(
                    &content.content,
                ))),
                404 => Err(ParameterValueError::FqnOrJmesPathNotFound(extract_details(
                    &content.content,
                ))),
                _ => Err(ParameterValueError::CreateError(response.unwrap_err())),
            },
            Err(e) => Err(ParameterValueError::CreateError(e)),
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
    ) -> Result<String, ParameterValueError> {
        let dynamic = fqn.is_some() || jmes_path.is_some();
        let value_update = PatchedValue {
            url: None,
            id: None,
            environment: None,
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
                400 => Err(ParameterValueError::InvalidFqnOrJmesPath(extract_details(
                    &content.content,
                ))),
                404 => Err(ParameterValueError::FqnOrJmesPathNotFound(extract_details(
                    &content.content,
                ))),
                _ => Err(ParameterValueError::UpdateError(response.unwrap_err())),
            },
            Err(e) => Err(ParameterValueError::UpdateError(e)),
        }
    }
}
