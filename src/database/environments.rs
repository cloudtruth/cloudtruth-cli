use crate::database::{extract_details, OpenApiConfig, PAGE_SIZE};

use crate::database::{EnvironmentDetails, EnvironmentTag};
use cloudtruth_restapi::apis::environments_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{EnvironmentCreate, PatchedEnvironment, PatchedTag, TagCreate};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fmt::Formatter;

pub struct Environments {}

/// This is used to map from an Environment's URL to the Name.
pub type EnvironmentUrlMap = HashMap<String, String>;

#[derive(Debug)]
pub enum EnvironmentError {
    ListError(Error<EnvironmentsListError>),
    DeleteError(Error<EnvironmentsDestroyError>),
    AuthError(String),
    DeleteNotAllowed(String),
    NotFound(String),
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentError::AuthError(msg) => {
                write!(f, "Not Authenticated: {}", msg)
            }
            EnvironmentError::DeleteNotAllowed(msg) => {
                write!(f, "Delete not allowed: {}", msg)
            }
            EnvironmentError::DeleteError(e) => {
                write!(f, "{}", e.to_string())
            }
            EnvironmentError::ListError(e) => {
                write!(f, "{}", e.to_string())
            }
            EnvironmentError::NotFound(name) => {
                write!(f, "Did not find environment '{}'", name)
            }
        }
    }
}

impl error::Error for EnvironmentError {}

/// The `BadRequest` content seems to be a list (instead of structured data like many other
/// `ResponseError` cases). This handles what appears to be a list of string, instead of structured
/// data handled in `extract_details()`.
fn bad_request_details(content: &str) -> String {
    content
        .trim_start_matches("[\"")
        .trim_end_matches("\"]")
        .to_string()
}

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    /// Use the environment URL to get the corresponding name.
    pub fn get_name_from_url(&self, rest_cfg: &OpenApiConfig, url: &str) -> String {
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
    pub fn get_url_name_map(&self, rest_cfg: &OpenApiConfig) -> EnvironmentUrlMap {
        let response = environments_list(rest_cfg, None, None, PAGE_SIZE, None);
        let mut result: EnvironmentUrlMap = EnvironmentUrlMap::new();
        if let Ok(list) = response {
            if let Some(environments) = list.results {
                for env in environments {
                    result.insert(env.url, env.name);
                }
            }
        }
        result
    }

    /// Uses the URL/name map to get the identifier for the provided environment name.
    pub fn id_from_map(
        &self,
        name: &str,
        url_map: &EnvironmentUrlMap,
    ) -> Result<String, EnvironmentError> {
        for (k, v) in url_map {
            if name == v.as_str() {
                let segments: Vec<&str> = k.rsplit('/').collect();
                for seg in segments {
                    if !seg.is_empty() {
                        return Ok(seg.to_string());
                    }
                }
            }
        }
        Err(EnvironmentError::NotFound(name.to_string()))
    }

    /// Gets the `EnvironmentDetails` for the provided name.
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        env_name: &str,
    ) -> Result<Option<EnvironmentDetails>, EnvironmentError> {
        let response = environments_list(rest_cfg, Some(env_name), None, PAGE_SIZE, None);

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        let env = &list[0];
                        let mut details = EnvironmentDetails::from(env);
                        details.parent_name =
                            self.get_name_from_url(rest_cfg, details.parent_url.as_str());
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

    /// Gets the environment's identifier for the provided name.
    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        env_name: &str,
    ) -> Result<Option<String>, EnvironmentError> {
        if let Some(details) = self.get_details_by_name(rest_cfg, env_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    pub fn get_environment_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<EnvironmentDetails>, EnvironmentError> {
        let response = environments_list(rest_cfg, None, None, PAGE_SIZE, None);

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    let mut env_info: Vec<EnvironmentDetails> = Vec::new();
                    let mut url_map: EnvironmentUrlMap = EnvironmentUrlMap::new();
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
        rest_cfg: &OpenApiConfig,
        env_name: &str,
        description: Option<&str>,
        parent_url: &str,
    ) -> Result<Option<String>, Error<EnvironmentsCreateError>> {
        let new_env = EnvironmentCreate {
            name: env_name.to_string(),
            description: description.map(String::from),
            parent: Some(parent_url.to_string()),
        };
        let response = environments_create(rest_cfg, new_env)?;
        // return the id of the new environment (likely same as the old)
        Ok(Some(response.id))
    }

    pub fn delete_environment(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: String,
    ) -> Result<String, EnvironmentError> {
        let response = environments_destroy(rest_cfg, &environment_id);
        match response {
            Ok(_) => Ok(environment_id),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                400 => Err(EnvironmentError::DeleteNotAllowed(bad_request_details(
                    &content.content,
                ))),
                409 => Err(EnvironmentError::DeleteNotAllowed(extract_details(
                    &content.content,
                ))),
                _ => Err(EnvironmentError::DeleteError(response.unwrap_err())),
            },
            Err(e) => Err(EnvironmentError::DeleteError(e)),
        }
    }

    pub fn update_environment(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        environment_name: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<EnvironmentsPartialUpdateError>> {
        let env = PatchedEnvironment {
            url: None,
            id: None,
            name: Some(environment_name.to_string()),
            description: description.map(String::from),
            parent: None,
            created_at: None,
            modified_at: None,
        };
        let response = environments_partial_update(rest_cfg, environment_id, Some(env))?;
        Ok(Some(response.id))
    }

    /// Converts a list of `EnvironmentDetails` to an `EnvironmentUrlMap`.
    ///
    /// This is used when you already have to get all the details, but want to get a simple
    /// Url -> Name map for resolving environments by URL.
    pub fn details_to_map(&self, details: &[EnvironmentDetails]) -> EnvironmentUrlMap {
        let mut result = EnvironmentUrlMap::new();
        for d in details {
            result.insert(d.url.clone(), d.name.clone());
        }
        result
    }

    pub fn get_env_tags(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
    ) -> Result<Vec<EnvironmentTag>, Error<EnvironmentsTagsListError>> {
        let response = environments_tags_list(
            rest_cfg,
            environment_id,
            None,
            None,
            None,
            None,
            None,
            PAGE_SIZE,
            None,
            None,
            None,
        )?;
        let mut result: Vec<EnvironmentTag> = vec![];
        if let Some(list) = response.results {
            for ref entry in list {
                result.push(EnvironmentTag::from(entry));
            }
        }
        Ok(result)
    }

    pub fn get_tag_id(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
    ) -> Result<Option<String>, Error<EnvironmentsTagsListError>> {
        let response = environments_tags_list(
            rest_cfg,
            environment_id,
            None,
            Some(tag_name),
            None,
            None,
            None,
            PAGE_SIZE,
            None,
            None,
            None,
        )?;
        let mut result = None;
        if let Some(list) = response.results {
            if !list.is_empty() {
                result = Some(list[0].id.clone());
            }
        }
        Ok(result)
    }

    pub fn get_tag_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
    ) -> Result<Option<EnvironmentTag>, Error<EnvironmentsTagsListError>> {
        let response = environments_tags_list(
            rest_cfg,
            environment_id,
            None,
            Some(tag_name),
            None,
            None,
            None,
            PAGE_SIZE,
            None,
            None,
            None,
        )?;
        if let Some(list) = response.results {
            if !list.is_empty() {
                Ok(Some(EnvironmentTag::from(&list[0])))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn create_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
        description: Option<&str>,
        timestamp: Option<String>,
    ) -> Result<String, Error<EnvironmentsTagsCreateError>> {
        let tag_create = TagCreate {
            name: tag_name.to_string(),
            description: description.map(String::from),
            timestamp,
        };
        let response = environments_tags_create(rest_cfg, environment_id, tag_create)?;
        Ok(response.id)
    }

    pub fn update_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_id: &str,
        tag_name: &str,
        description: Option<&str>,
        timestamp: Option<String>,
    ) -> Result<EnvironmentTag, Error<EnvironmentsTagsPartialUpdateError>> {
        let tag_update = PatchedTag {
            url: None,
            id: None,
            name: Some(tag_name.to_string()),
            description: description.map(String::from),
            timestamp,
            usage: None,
        };
        let response =
            environments_tags_partial_update(rest_cfg, environment_id, tag_id, Some(tag_update))?;
        Ok(EnvironmentTag::from(&response))
    }

    pub fn delete_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_id: &str,
    ) -> Result<String, Error<EnvironmentsTagsDestroyError>> {
        let _ = environments_tags_destroy(rest_cfg, environment_id, tag_id)?;
        Ok(tag_id.to_string())
    }
}
