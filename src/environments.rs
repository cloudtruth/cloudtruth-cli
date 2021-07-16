use crate::openapi::{extract_details, open_api_config};

use cloudtruth_restapi::apis::environments_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{Environment, EnvironmentCreate, PatchedEnvironment};
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Formatter};

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

#[derive(Debug)]
pub enum EnvironmentError {
    ListError(Error<EnvironmentsListError>),
    AuthError(String),
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentError::AuthError(msg) => {
                write!(f, "Not Authenticated: {}", msg)
            }
            EnvironmentError::ListError(e) => {
                write!(f, "{}", e.to_string())
            }
        }
    }
}

impl error::Error for EnvironmentError {}

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
    ) -> Result<Option<EnvironmentDetails>, EnvironmentError> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, Some(env_name), None, None);

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        let env = &list[0];
                        let mut details = EnvironmentDetails::from(env);
                        details.parent_name = self.get_name_from_url(details.parent_url.as_str());
                        Ok(Some(details))
                    }
                }
                None => Ok(None),
            },
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                401 => Err(EnvironmentError::AuthError(extract_details(
                    &content.content,
                ))),
                403 => Err(EnvironmentError::AuthError(extract_details(
                    &content.content,
                ))),
                _ => Err(EnvironmentError::ListError(response.unwrap_err())),
            },
            Err(e) => Err(EnvironmentError::ListError(e)),
        }
    }

    pub fn get_id(&self, env_name: &str) -> Result<Option<String>, EnvironmentError> {
        if let Some(details) = self.get_details_by_name(env_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    pub fn get_environment_details(&self) -> Result<Vec<EnvironmentDetails>, EnvironmentError> {
        let rest_cfg = open_api_config();
        let response = environments_list(&rest_cfg, None, None, None);

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    let mut env_info: Vec<EnvironmentDetails> = Vec::new();
                    let mut url_map: HashMap<String, String> = HashMap::new(); // maps URL to name
                    for env in list {
                        let details = EnvironmentDetails::from(&env);
                        url_map.insert(details.url.clone(), details.name.clone());
                        env_info.push(details);
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
                None => Ok(vec![]),
            },
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                401 => Err(EnvironmentError::AuthError(extract_details(
                    &content.content,
                ))),
                403 => Err(EnvironmentError::AuthError(extract_details(
                    &content.content,
                ))),
                _ => Err(EnvironmentError::ListError(response.unwrap_err())),
            },
            Err(e) => Err(EnvironmentError::ListError(e)),
        }
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
        environment_id: &str,
        environment_name: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<EnvironmentsPartialUpdateError>> {
        let rest_cfg = open_api_config();
        let env = PatchedEnvironment {
            url: None,
            id: None,
            name: Some(environment_name.to_string()),
            description: description.map(String::from),
            parent: None,
            created_at: None,
            modified_at: None,
        };
        let response = environments_partial_update(&rest_cfg, environment_id, Some(env))?;
        Ok(Some(response.id))
    }
}
