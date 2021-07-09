use crate::environments::Environments;
use crate::openapi::open_api_config;
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::{Parameter, ParameterCreate, PatchedValue, Value, ValueCreate};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::result::Result;
use std::str::FromStr;

pub struct Parameters {}

const MASK_SECRETS: Option<bool> = Some(false); // TODO: tie usage to a new parameter

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
        value: Some("----".to_owned()),
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
            Ok(None) // TODO: this should return an error
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
            MASK_SECRETS,
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
        let parameters = self.get_parameter_details(proj_id, env_id)?;
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
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list = self.get_parameter_unresolved_details(proj_id, env_id)?;

        // now, resolve the source URL to the source environment name
        let environment = Environments::new();
        let url_map = environment.get_url_name_map();
        for details in &mut list {
            details.env_name = url_map.get(&details.env_url).unwrap().clone();
        }
        Ok(list)
    }

    /// This internal function gets the `ParameterDetails`, but does not resolve the `source` from
    /// the URL to the name.
    fn get_parameter_unresolved_details(
        &self,
        proj_id: &str,
        env_id: &str,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let mut list: Vec<ParameterDetails> = Vec::new();
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            proj_id,
            Some(env_id),
            MASK_SECRETS,
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

    fn create_parameter_value(
        &self,
        proj_id: &str,
        env_id: &str,
        param_id: String,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsParametersValuesCreateError>> {
        let rest_cfg = open_api_config();
        let dynamic = value.is_none() || fqn.is_some();

        let value_new = ValueCreate {
            environment: env_id.to_string(),
            dynamic: Some(dynamic),
            static_value: value.map(|v| v.to_string()),
            dynamic_fqn: fqn.map(|v| v.to_string()),
            dynamic_filter: jmes_path.map(|v| v.to_string()),
        };
        let response = projects_parameters_values_create(
            &rest_cfg,
            param_id.as_str(),
            proj_id,
            value_new,
            None,
        );
        if let Ok(result) = response {
            Ok(Some(result.id))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_parameter(
        &self,
        proj_id: &str,
        env_id: &str,
        key_name: &str,
        value: Option<&str>,
        description: Option<&str>,
        secret: Option<bool>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsParametersUpdateError>> {
        let mut result: Option<String> = None;
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            proj_id,
            Some(env_id),
            MASK_SECRETS,
            Some(key_name),
            None,
            None,
        );
        if let Ok(paged_results) = response {
            if let Some(list) = paged_results.results {
                let mut param: Parameter = list[0].clone();
                let mut param_changed = false;
                if let Some(desc_str) = description {
                    param.description = Some(desc_str.to_string());
                    param_changed = true;
                }
                if secret.is_some() {
                    param.secret = secret;
                    param_changed = true;
                }

                let dynamic = value.is_none() || fqn.is_some();
                let env_value: &Value = param.values.values().next().unwrap();

                if value.is_none() && fqn.is_none() && jmes_path.is_none() {
                    // nothing to set here, no need for updates
                } else if env_value.environment.as_str() == env_id {
                    // update
                    let value_up = PatchedValue {
                        url: None,
                        id: None,
                        environment: None,
                        parameter: None,
                        dynamic: Some(dynamic),
                        static_value: value.map(|v| v.to_string()),
                        dynamic_fqn: fqn.map(|v| v.to_string()),
                        dynamic_filter: jmes_path.map(|v| v.to_string()),
                        value: None,
                        created_at: None,
                        modified_at: None,
                    };
                    let response = projects_parameters_values_partial_update(
                        &rest_cfg,
                        env_value.id.as_str(),
                        param.id.as_str(),
                        proj_id,
                        None,
                        Some(value_up),
                    );
                    if let Ok(value) = response {
                        result = Some(format!("{}/{}", param.id, value.id));
                    }
                } else {
                    let response = self.create_parameter_value(
                        proj_id,
                        env_id,
                        param.id.clone(),
                        value,
                        fqn,
                        jmes_path,
                    );
                    if let Ok(Some(value_id)) = response {
                        result = Some(format!("{}/{}", param.id, value_id));
                    }
                }

                if param_changed {
                    let param_id = param.id.clone();
                    projects_parameters_update(&rest_cfg, param_id.as_str(), proj_id, param)?;
                }
            } else {
                let param_new = ParameterCreate {
                    name: key_name.to_string(),
                    description: description.map(|x| x.to_string()),
                    secret,
                };
                let response = projects_parameters_create(&rest_cfg, proj_id, param_new);
                if let Ok(param) = response {
                    let response = self.create_parameter_value(
                        proj_id,
                        env_id,
                        param.id.clone(),
                        value,
                        fqn,
                        jmes_path,
                    );
                    if let Ok(Some(value_id)) = response {
                        result = Some(format!("{}/{}", param.id, value_id))
                    }
                }
            }
        }

        Ok(result)
    }
}
