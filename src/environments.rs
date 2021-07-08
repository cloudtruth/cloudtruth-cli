use crate::openapi::open_api_config;

use cloudtruth_restapi::apis::environments_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::{Environment, EnvironmentCreate, PatchedEnvironment};
use std::collections::HashMap;

pub struct Environments {}

#[derive(Debug)]
pub struct EnvironmentDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub parent_url: String,
    pub parent_name: String,
}

/// Converts the OpenApi `Environment` reference into a CloudTruth `EnvironmentDetails` object.
///
/// The `parent_name` is filled in later, so it can be done with a map of URLs to names.
impl From<&Environment> for EnvironmentDetails {
    fn from(api_env: &Environment) -> Self {
        EnvironmentDetails {
            id: api_env.id.clone(),
            url: api_env.url.clone(),
            name: api_env.name.clone(),
            description: api_env.description.clone().unwrap_or_default(),
            parent_url: api_env.parent.clone().unwrap_or_default(),
            parent_name: "".to_owned(),
        }
    }
}

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    /// Use the environment URL to get the corresponding name.
    pub fn get_name_from_url(&self, url: &str) -> String {
        let rest_cfg = open_api_config();
        let id = url
            .split('/')
            .filter(|&x| !x.is_empty())
            .last()
            .unwrap_or_default();
        if id.is_empty() {
            "".to_owned()
        } else {
            let response = environments_retrieve(rest_cfg, id);
            if let Ok(environment) = response {
                environment.name
            } else {
                "".to_owned()
            }
        }
    }

    /// This provides a means to get an entire list of environment URLs to names.
    pub fn get_url_name_map(&self) -> HashMap<String, String> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, None, None, None);
        let mut result: HashMap<String, String> = HashMap::new();
        if let Ok(list) = response {
            if let Some(environments) = list.results {
                for env in environments {
                    result.insert(env.url, env.name);
                }
            }
        }
        result
    }

    pub fn get_details_by_name(
        &self,
        env_name: &str,
    ) -> Result<Option<EnvironmentDetails>, Error<EnvironmentsListError>> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, Some(env_name), None, None)?;

        if let Some(environments) = response.results {
            if environments.is_empty() {
                Ok(None)
            } else {
                // TODO: handle more than one??
                let env = &environments[0];
                let mut details = EnvironmentDetails::from(env);
                details.parent_name = self.get_name_from_url(details.parent_url.as_str());
                Ok(Some(details))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_id(&self, env_name: &str) -> Result<Option<String>, Error<EnvironmentsListError>> {
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
        let mut url_map: HashMap<String, String> = HashMap::new(); // maps URL to name

        if let Some(resp_env) = response.results {
            for env in resp_env {
                let details = EnvironmentDetails::from(&env);
                url_map.insert(details.url.clone(), details.name.clone());
                env_info.push(details);
            }
        }

        // now, fill in the names
        for details in &mut env_info {
            if !details.parent_url.is_empty() {
                details.parent_name = url_map.get(&details.parent_url).unwrap().clone();
            }
        }
        env_info.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(env_info)
    }

    pub fn create_environment(
        &self,
        env_name: &str,
        description: Option<&str>,
        parent_url: &str,
    ) -> Result<Option<String>, Error<EnvironmentsCreateError>> {
        let rest_cfg = open_api_config();
        let new_env = EnvironmentCreate {
            name: env_name.to_string(),
            description: description.map(String::from),
            parent: Some(parent_url.to_string()),
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
            created_at: None,
            modified_at: None,
        };
        let response = environments_partial_update(&rest_cfg, environment_id.as_str(), Some(env))?;
        Ok(Some(response.id))
    }
}
