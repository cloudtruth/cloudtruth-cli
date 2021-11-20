use crate::database::{
    auth_details, extract_details, last_from_url, page_size, response_message, EnvironmentDetails,
    EnvironmentError, EnvironmentTag, OpenApiConfig, NO_PAGE_COUNT, NO_PAGE_SIZE,
};
use cloudtruth_restapi::apis::environments_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    EnvironmentCreate, PatchedEnvironment, PatchedTagUpdate, TagCreate,
};
use std::collections::HashMap;

const NO_DESC_CONTAINS: Option<&str> = None;
const NO_NAME_CONTAINS: Option<&str> = None;
const NO_ORDERING: Option<&str> = None;
const NO_PARENT_NAME: Option<&str> = None;
const NO_PARENT_CONTAINS: Option<&str> = None;

pub struct Environments {}

/// This is used to map from an Environment's URL to the Name.
pub type EnvironmentUrlMap = HashMap<String, String>;

fn response_error(status: &reqwest::StatusCode, content: &str) -> EnvironmentError {
    EnvironmentError::ResponseError(response_message(status, content))
}

fn auth_error(content: &str) -> EnvironmentError {
    EnvironmentError::Authentication(auth_details(content))
}

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    /// Use the environment URL to get the corresponding name.
    pub fn get_name_from_url(&self, rest_cfg: &OpenApiConfig, url: &str) -> String {
        let id = last_from_url(url);
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
        let mut result: EnvironmentUrlMap = EnvironmentUrlMap::new();
        let mut page_count = 1;
        loop {
            let response = environments_list(
                rest_cfg,
                NO_DESC_CONTAINS,
                None,
                NO_NAME_CONTAINS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                NO_PARENT_NAME,
                NO_PARENT_CONTAINS,
            );
            if let Ok(data) = response {
                if let Some(environments) = data.results {
                    for env in environments {
                        result.insert(env.url, env.name);
                    }
                    page_count += 1;
                }
                if data.next.is_none() {
                    break;
                }
            } else {
                break;
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
        let response = environments_list(
            rest_cfg,
            NO_DESC_CONTAINS,
            Some(env_name),
            NO_NAME_CONTAINS,
            NO_ORDERING,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
            NO_PARENT_NAME,
            NO_PARENT_CONTAINS,
        );

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
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
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
        let mut env_info: Vec<EnvironmentDetails> = Vec::new();
        let mut url_map: EnvironmentUrlMap = EnvironmentUrlMap::new();
        let mut page_count = 1;
        loop {
            let response = environments_list(
                rest_cfg,
                NO_DESC_CONTAINS,
                None,
                NO_NAME_CONTAINS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                NO_PARENT_NAME,
                NO_PARENT_CONTAINS,
            );
            match response {
                Ok(data) => {
                    if let Some(list) = data.results {
                        for env in list {
                            let details = EnvironmentDetails::from(&env);
                            url_map.insert(details.url.clone(), details.name.clone());
                            env_info.push(details);
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => match content.status.as_u16() {
                    401 => return Err(auth_error(&content.content)),
                    403 => return Err(auth_error(&content.content)),
                    _ => return Err(response_error(&content.status, &content.content)),
                },
                Err(e) => return Err(EnvironmentError::UnhandledError(e.to_string())),
            }
        } // loop

        // now, fill in the names
        let default_envname = "".to_string();
        for details in &mut env_info {
            if !details.parent_url.is_empty() {
                details.parent_name = url_map
                    .get(&details.parent_url)
                    .unwrap_or(&default_envname)
                    .clone();
            }
        }
        env_info.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(env_info)
    }

    pub fn create_environment(
        &self,
        rest_cfg: &OpenApiConfig,
        env_name: &str,
        description: Option<&str>,
        parent_url: &str,
    ) -> Result<Option<String>, EnvironmentError> {
        let new_env = EnvironmentCreate {
            name: env_name.to_string(),
            description: description.map(String::from),
            parent: Some(parent_url.to_string()),
        };
        let response = environments_create(rest_cfg, new_env);
        match response {
            Ok(env) => Ok(Some(env.id)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
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
                400 => Err(EnvironmentError::DeleteNotAllowed(extract_details(
                    &content.content,
                ))),
                409 => Err(EnvironmentError::DeleteNotAllowed(extract_details(
                    &content.content,
                ))),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
    }

    pub fn update_environment(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        environment_name: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, EnvironmentError> {
        let env = PatchedEnvironment {
            url: None,
            id: None,
            name: Some(environment_name.to_string()),
            description: description.map(String::from),
            parent: None,
            children: None,
            created_at: None,
            modified_at: None,
        };
        let response = environments_partial_update(rest_cfg, environment_id, Some(env));
        match response {
            Ok(env) => Ok(Some(env.id)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
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
    ) -> Result<Vec<EnvironmentTag>, EnvironmentError> {
        let mut result: Vec<EnvironmentTag> = vec![];
        let mut page_count = 1;

        loop {
            let response = environments_tags_list(
                rest_cfg,
                environment_id,
                None,
                None,
                None,
                None,
                Some(page_count),
                page_size(rest_cfg),
                None,
                None,
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(list) = data.results {
                        for ref entry in list {
                            result.push(EnvironmentTag::from(entry));
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
                Err(e) => return Err(EnvironmentError::UnhandledError(e.to_string())),
            }
        } // loop
        Ok(result)
    }

    pub fn get_tag_id(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
    ) -> Result<Option<String>, EnvironmentError> {
        let details = self.get_tag_by_name(rest_cfg, environment_id, tag_name)?;
        Ok(details.map(|d| d.id))
    }

    pub fn get_tag_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
    ) -> Result<Option<EnvironmentTag>, EnvironmentError> {
        let response = environments_tags_list(
            rest_cfg,
            environment_id,
            None,
            Some(tag_name),
            None,
            None,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
            None,
            None,
            None,
        );
        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if !list.is_empty() {
                        Ok(Some(EnvironmentTag::from(&list[0])))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_tag_time(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        environment_name: &str,
        tag_name: &str,
    ) -> Result<String, EnvironmentError> {
        let result = self.get_tag_by_name(rest_cfg, environment_id, tag_name)?;
        match result {
            Some(details) => Ok(details.timestamp),
            _ => Err(EnvironmentError::TagNotFound(
                environment_name.to_string(),
                tag_name.to_string(),
            )),
        }
    }

    pub fn create_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_name: &str,
        description: Option<&str>,
        timestamp: Option<String>,
    ) -> Result<String, EnvironmentError> {
        let tag_create = TagCreate {
            name: tag_name.to_string(),
            description: description.map(String::from),
            timestamp,
        };
        let response = environments_tags_create(rest_cfg, environment_id, tag_create);
        match response {
            Ok(tag) => Ok(tag.id),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
    }

    pub fn update_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_id: &str,
        tag_name: &str,
        description: Option<&str>,
        timestamp: Option<String>,
    ) -> Result<(), EnvironmentError> {
        let tag_update = PatchedTagUpdate {
            id: None,
            name: Some(tag_name.to_string()),
            description: description.map(String::from),
            timestamp,
        };
        let response =
            environments_tags_partial_update(rest_cfg, environment_id, tag_id, Some(tag_update));
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_env_tag(
        &self,
        rest_cfg: &OpenApiConfig,
        environment_id: &str,
        tag_id: &str,
    ) -> Result<String, EnvironmentError> {
        let response = environments_tags_destroy(rest_cfg, environment_id, tag_id);
        match response {
            Ok(_) => Ok(tag_id.to_string()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(EnvironmentError::UnhandledError(e.to_string())),
        }
    }
}
