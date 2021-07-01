use crate::openapi::open_api_config;
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::{Parameter, ParameterCreate, PatchedValue, Value, ValueCreate};
use std::collections::HashMap;
use std::result::Result;
use std::str::FromStr;

pub struct Parameters {}

const MASK_SECRETS: Option<bool> = Some(false); // TODO: tie usage to a new parameter

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
    pub source: String,
    pub dynamic: bool,
    pub fqn: String,
    pub jmes_path: String,
}

impl From<&Parameter> for ParameterDetails {
    fn from(api_param: &Parameter) -> Self {
        let env_value: &Value = &api_param.values[0];

        ParameterDetails {
            id: api_param.id.clone(),
            key: api_param.name.clone(),
            secret: api_param.secret.unwrap_or(false),
            description: api_param.description.clone().unwrap_or_default(),

            val_id: env_value.id.clone(),
            value: env_value.value.clone().unwrap_or_default(),
            source: env_value.environment.clone(),
            dynamic: env_value.dynamic.unwrap_or(false),
            fqn: env_value.dynamic_fqn.clone().unwrap_or_default(),
            jmes_path: env_value.dynamic_filter.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub enum ParamExportFormat {
    DOCKER,
    DOTENV,
    SHELL,
}

/// Converts to ParamExportFormat from a &str.
impl FromStr for ParamExportFormat {
    type Err = ();

    fn from_str(input: &str) -> Result<ParamExportFormat, Self::Err> {
        match input {
            "docker" => Ok(ParamExportFormat::DOCKER),
            "dotenv" => Ok(ParamExportFormat::DOTENV),
            "shell" => Ok(ParamExportFormat::SHELL),
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
        param_id: String,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        let rest_cfg = open_api_config();
        projects_parameters_destroy(&rest_cfg, param_id.as_str(), "")?;
        Ok(Some(param_id))
    }

    /// Deletes the specified parameter from the specified project/environment.
    ///
    /// On success, it returns the ID of the deleted value. On failure, it returns an Error
    /// with more failure information.
    pub fn delete_parameter(
        &self,
        proj_name: Option<String>,
        env_name: Option<&str>,
        key_name: &str,
    ) -> Result<Option<String>, Error<ProjectsParametersDestroyError>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let response = self.get_details_by_name(env_name, proj_name, key_name);

        if let Ok(Some(details)) = response {
            self.delete_param_by_id(details.id)
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
        _project_name: Option<String>,
        _environment_name: Option<&str>,
        _options: ParamExportOptions,
    ) -> Result<Option<String>, Error<ProjectsParametersListError>> {
        // let rest_cfg = open_api_config();

        // TODO: implement this
        Ok(None)
    }

    /// Fetches the `ParameterDetails` for the specified project/environment/key_name.
    ///
    /// It will return `None` if the parameter does not exist. Other errors will be returned
    /// if project/environments are not found.
    pub fn get_details_by_name(
        &self,
        env_name: Option<&str>,
        proj_name: Option<String>,
        key_name: &str,
    ) -> Result<Option<ParameterDetails>, Error<ProjectsParametersListError>> {
        let rest_cfg = open_api_config();
        // TODO: project_pk id or name? environment id or name?
        let response = projects_parameters_list(
            &rest_cfg,
            proj_name.unwrap().as_str(),
            env_name,
            MASK_SECRETS,
            Some(key_name),
            None,
            None,
        )?;
        if let Some(parameters) = response.results {
            // TODO: handle more than one??
            let param = &parameters[0];
            Ok(Some(ParameterDetails::from(param)))
        } else {
            Ok(None)
        }
    }

    /// Fetches a "dictionary" of environment variable name/values for the specified project and
    /// environment.
    pub fn get_parameter_values(
        &self,
        env_id: Option<String>,
        proj_name: Option<String>,
    ) -> Result<HashMap<String, String>, Error<ProjectsParametersListError>> {
        let parameters = self.get_parameter_details(env_id, proj_name)?;
        let mut env_vars = HashMap::new();

        for param in parameters {
            env_vars.insert(param.key, param.value);
        }
        Ok(env_vars)
    }

    /// Fetches the `ParameterDetails` for the specified project and environment.
    pub fn get_parameter_details(
        &self,
        env_id: Option<String>,
        proj_name: Option<String>,
    ) -> Result<Vec<ParameterDetails>, Error<ProjectsParametersListError>> {
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            proj_name.unwrap_or_default().as_str(),
            env_id.as_deref(),
            MASK_SECRETS,
            None,
            None,
            None,
        )?;
        let mut list: Vec<ParameterDetails> = Vec::new();
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
        proj_id: Option<String>,
        param_id: String,
        env_name: Option<&str>,
        value: Option<&str>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsParametersValuesCreateError>> {
        let rest_cfg = open_api_config();
        let dynamic = value.is_none() || fqn.is_some();

        let value_new = ValueCreate {
            environment: env_name.unwrap().to_string(),
            dynamic: Some(dynamic),
            static_value: value.map(|v| v.to_string()),
            dynamic_fqn: fqn.map(|v| v.to_string()),
            dynamic_filter: jmes_path.map(|v| v.to_string()),
        };
        let response = projects_parameters_values_create(
            &rest_cfg,
            param_id.as_str(),
            proj_id.unwrap().as_str(),
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
        proj_id: Option<String>,
        env_name: Option<&str>,
        key_name: &str,
        value: Option<&str>,
        description: Option<&str>,
        secret: Option<bool>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsParametersUpdateError>> {
        let mut result: Option<String> = None;
        let project_id = proj_id.as_ref().unwrap().as_str();
        let rest_cfg = open_api_config();
        let response = projects_parameters_list(
            &rest_cfg,
            project_id,
            env_name,
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
                let env_value: &Value = &param.values[0];

                if value.is_none() && fqn.is_none() && jmes_path.is_none() {
                    // nothing to set here, no need for updates
                } else if env_value.environment.as_str() == env_name.unwrap() {
                    // update
                    let value_up = PatchedValue {
                        url: None,
                        id: None,
                        environment: None,
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
                        project_id,
                        None,
                        Some(value_up),
                    );
                    if let Ok(value) = response {
                        result = Some(format!("{}/{}", param.id, value.id));
                    }
                } else {
                    let response = self.create_parameter_value(
                        proj_id.clone(),
                        param.id.clone(),
                        env_name,
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
                    projects_parameters_update(&rest_cfg, param_id.as_str(), project_id, param)?;
                }
            } else {
                let param_new = ParameterCreate {
                    name: key_name.to_string(),
                    description: description.map(|x| x.to_string()),
                    secret,
                };
                let response = projects_parameters_create(&rest_cfg, project_id, param_new);
                if let Ok(param) = response {
                    let response = self.create_parameter_value(
                        proj_id,
                        param.id.clone(),
                        env_name,
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
