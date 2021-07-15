use crate::environments::Environments;
use crate::openapi::{extract_details, open_api_config};
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
}

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
        static_value: None,
        value: Some("—".to_owned()),
        created_at: "".to_owned(),
        modified_at: "".to_owned(),
    })
}

impl From<&Parameter> for ParameterDetails {
    fn from(api_param: &Parameter) -> Self {
        let env_value = api_param
            .values
            .values()
            .next()
            .unwrap_or_else(|| default_param_value());

        ParameterDetails {
            id: api_param.id.clone(),
            key: api_param.name.clone(),
            secret: api_param.secret.unwrap_or(false),
            description: api_param.description.clone().unwrap_or_default(),

            val_id: env_value.id.clone(),
            value: env_value.value.clone().unwrap_or_default(),
            env_url: env_value.environment.clone(),
            env_name: "".to_owned(),
            dynamic: env_value.dynamic.unwrap_or(false),
            fqn: env_value.dynamic_fqn.clone().unwrap_or_default(),
            jmes_path: env_value.dynamic_filter.clone().unwrap_or_default(),
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
    fn delete_param_by_id(
        &self,
        proj_id: &str,
        param_id: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        let rest_cfg = open_api_config();
        projects_parameters_destroy(&rest_cfg, param_id, proj_id)?;
        Ok(Some(param_id.to_string()))
    }

    /// Deletes the specified parameter from the specified project/environment.
    ///
    /// On success, it returns the ID of the deleted value. On failure, it returns an Error
    /// with more failure information.
    pub fn delete_parameter(
        &self,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let response = self.get_details_by_name(proj_id, env_id, key_name);

        if let Ok(Some(details)) = response {
            self.delete_param_by_id(proj_id, details.id.as_str())
        } else {
            Ok(None)
        }
    }

    /// Deletes the "override" for the specified environment.
    pub fn delete_parameter_value(
        &self,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersValuesDestroyError>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let response = self.get_details_by_name(proj_id, env_id, key_name);

        if let Ok(Some(details)) = response {
            if details.env_url.contains(env_id) {
                let rest_cfg = open_api_config();
                projects_parameters_values_destroy(
                    &rest_cfg,
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
        proj_id: &str,
        env_id: &str,
        options: ParamExportOptions,
    ) -> Result<Option<String>, Error<ProjectsParameterExportListError>> {
        let rest_cfg = open_api_config();
        let out_fmt = format!("{:?}", options.format).to_lowercase();
        let mask_secrets = Some(!options.secrets.unwrap_or(false));
        let export = projects_parameter_export_list(
            &rest_cfg,
            proj_id,
            options.contains.as_deref(),
            options.ends_with.as_deref(),
            Some(env_id),
            options.export,
            mask_secrets,
            Some(out_fmt.as_str()),
            None,
            options.starts_with.as_deref(),
            None,
        )?;
        Ok(Some(export.body))
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
        proj_id: &str,
        env_id: &str,
        key_name: &str,
    ) -> Result<Option<ParameterDetails>, Error<ProjectsParametersListError>> {
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            proj_id,
            Some(env_id),
            Some(false), // get secret value when querying a single parameter
            Some(key_name),
            None,
            None,
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
        proj_id: &str,
        env_id: &str,
    ) -> Result<HashMap<String, String>, Error<ProjectsParametersListError>> {
        let parameters = self.get_parameter_unresolved_details(proj_id, env_id, false)?;
        let mut env_vars = HashMap::new();

        for param in parameters {
            env_vars.insert(param.key, param.value);
        }
        Ok(env_vars)
    }

    /// Fetches the `ParameterDetails` for the specified project and environment.
    pub fn get_parameter_details(
        &self,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list = self.get_parameter_unresolved_details(proj_id, env_id, mask_secrets)?;

        // now, resolve the source URL to the source environment name
        let environments = Environments::new();
        let url_map = environments.get_url_name_map();
        let default_key = "".to_string();
        for details in &mut list {
            details.env_name = url_map
                .get(&details.env_url)
                .unwrap_or(&default_key)
                .clone();
        }
        Ok(list)
    }

    /// This internal function gets the `ParameterDetails`, but does not resolve the `source` from
    /// the URL to the name.
    fn get_parameter_unresolved_details(
        &self,
        proj_id: &str,
        env_id: &str,
        mask_secrets: bool,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list: Vec<ParameterDetails> = Vec::new();
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            proj_id,
            Some(env_id),
            Some(mask_secrets),
            None,
            None,
            None,
        )?;
        if let Some(parameters) = response.results {
            for param in parameters {
                list.push(ParameterDetails::from(&param));
            }
            list.sort_by(|l, r| l.key.cmp(&r.key));
        }
        Ok(list)
    }

    /// Creates the `Parameter` entry.
    ///
    /// There is no `Value` entry created as part of this -- it is just the `Parameter`.
    pub fn create_parameter(
        &self,
        proj_id: &str,
        key_name: &str,
        description: Option<&str>,
        secret: Option<bool>,
    ) -> Result<ParameterDetails, Error<ProjectsParametersCreateError>> {
        let rest_cfg = open_api_config();
        let param_new = ParameterCreate {
            name: key_name.to_string(),
            description: description.map(|x| x.to_string()),
            secret,
        };
        let api_param = projects_parameters_create(&rest_cfg, proj_id, param_new)?;
        Ok(ParameterDetails::from(&api_param))
    }

    /// Updates the `Parameter` entry.
    ///
    /// It does not touch any associated `Value` entries.
    pub fn update_parameter(
        &self,
        proj_id: &str,
        param_id: &str,
        key_name: &str,
        description: Option<&str>,
        secret: Option<bool>,
    ) -> Result<ParameterDetails, Error<ProjectsParametersPartialUpdateError>> {
        let rest_cfg = open_api_config();
        let param_update = PatchedParameter {
            url: None,
            id: None,
            name: Some(key_name.to_string()),
            description: description.map(String::from),
            secret,
            templates: None,
            uses_dynamic_values: None,
            values: None,
            created_at: None,
            modified_at: None,
        };
        let api_param =
            projects_parameters_partial_update(&rest_cfg, param_id, proj_id, Some(param_update))?;
        Ok(ParameterDetails::from(&api_param))
    }

    /// Creates a `Value` entry associated with the `Parameter` identified by the
    /// `proj_id`/`param_id`.
    pub fn create_parameter_value(
        &self,
        proj_id: &str,
        env_id: &str,
        param_id: &str,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<String, ParameterValueError> {
        let rest_cfg = open_api_config();
        let dynamic = value.is_none() || fqn.is_some();
        let value_create = ValueCreate {
            environment: env_id.to_string(),
            dynamic: Some(dynamic),
            static_value: value.map(|v| v.to_string()),
            dynamic_fqn: fqn.map(|v| v.to_string()),
            dynamic_filter: jmes_path.map(|v| v.to_string()),
        };
        let response =
            projects_parameters_values_create(&rest_cfg, param_id, proj_id, value_create, None);
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
    pub fn update_parameter_value(
        &self,
        proj_id: &str,
        param_id: &str,
        value_id: &str,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<String, ParameterValueError> {
        let rest_cfg = open_api_config();
        let dynamic = fqn.is_some() || jmes_path.is_some();
        let value_update = PatchedValue {
            url: None,
            id: None,
            environment: None,
            parameter: None,
            dynamic: Some(dynamic),
            dynamic_fqn: fqn.map(String::from),
            dynamic_filter: jmes_path.map(String::from),
            static_value: value.map(String::from),
            value: None,
            created_at: None,
            modified_at: None,
        };
        let response = projects_parameters_values_partial_update(
            &rest_cfg,
            value_id,
            param_id,
            proj_id,
            None,
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
