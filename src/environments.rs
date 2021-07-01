use crate::openapi::open_api_config;

use cloudtruth_restapi::apis::environments_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::{Environment, EnvironmentCreate, PatchedEnvironment};

pub struct Environments {}

#[derive(Debug)]
pub struct EnvironmentDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent: String,
}

/// Converts the OpenApi `Environment` reference into a CloudTruth `EnvironmentDetails` object.
impl From<&Environment> for EnvironmentDetails {
    fn from(api_env: &Environment) -> Self {
        let description = api_env.description.clone();
        let parent = api_env.parent.clone();
        EnvironmentDetails {
            id: api_env.id.clone(),
            name: api_env.name.clone(),
            description: description.unwrap_or_else(|| "".to_string()),
            parent: parent.unwrap_or_else(|| "".to_string()),
        }
    }
}

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_details_by_name(
        &self,
        env_name: Option<&str>,
    ) -> Result<Option<EnvironmentDetails>, Error<EnvironmentsListError>> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, env_name, None, None)?;

        if let Some(resp_env) = response.results {
            // TODO: handle more than one??
            let env = &resp_env[0];
            Ok(Some(EnvironmentDetails::from(env)))
        } else {
            Ok(None)
        }
    }

    pub fn get_id(
        &self,
        env_name: Option<&str>,
    ) -> Result<Option<String>, Error<EnvironmentsListError>> {
        if let Ok(Some(details)) = self.get_details_by_name(env_name) {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    pub fn get_environment_details(
        &self,
    ) -> Result<Vec<EnvironmentDetails>, Error<EnvironmentsListError>> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, None, None, None)?;
        let mut env_info: Vec<EnvironmentDetails> = Vec::new();

        if let Some(resp_env) = response.results {
            for env in resp_env {
                env_info.push(EnvironmentDetails::from(&env));
            }
        }
        env_info.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(env_info)
    }

    pub fn create_environment(
        &self,
        env_name: Option<&str>,
        description: Option<&str>,
        parent_id: String,
    ) -> Result<Option<String>, Error<EnvironmentsCreateError>> {
        let rest_cfg = open_api_config();
        let new_env = EnvironmentCreate {
            name: env_name.unwrap().to_string(),
            description: description.map(String::from),
            parent: Some(parent_id), // TODO: id or name?
        };
        let response = environments_create(&rest_cfg, new_env)?;
        // return the id of the new environment (likely same as the old)
        Ok(Some(response.id))
    }

    pub fn delete_environment(
        &self,
        environment_id: String,
    ) -> Result<String, Error<EnvironmentsDestroyError>> {
        let rest_cfg = open_api_config();
        environments_destroy(&rest_cfg, &environment_id)?;
        Ok(environment_id)
    }

    pub fn update_environment(
        &self,
        environment_id: String,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<EnvironmentsPartialUpdateError>> {
        // TODO: allow setting other fields (e.g. name)
        let rest_cfg = open_api_config();
        let env = PatchedEnvironment {
            url: None,
            id: None,
            name: None,
            description: description.map(String::from),
            parent: None,
        };
        let response = environments_partial_update(&rest_cfg, environment_id.as_str(), Some(env))?;
        Ok(Some(response.id))
    }
}
