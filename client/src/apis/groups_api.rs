/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

use reqwest;
use std::time::Instant;

use super::{configuration, Error};
use crate::apis::{handle_serde_error, ResponseContent};

/// struct for typed errors of method [`groups_add_create`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsAddCreateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_create`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsCreateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_destroy`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsDestroyError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_list`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsListError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_partial_update`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsPartialUpdateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_remove_create`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsRemoveCreateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_retrieve`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsRetrieveError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`groups_update`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GroupsUpdateError {
    UnknownValue(serde_json::Value),
}

/// Add a user to the group.
pub fn groups_add_create(
    configuration: &configuration::Configuration,
    id: &str,
    user: &str,
    group: crate::models::Group,
) -> Result<crate::models::Group, Error<GroupsAddCreateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/add/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("user", &user.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&group);

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsAddCreateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_create(
    configuration: &configuration::Configuration,
    group: crate::models::Group,
) -> Result<crate::models::Group, Error<GroupsCreateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/groups/", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&group);

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsCreateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_destroy(
    configuration: &configuration::Configuration,
    id: &str,
) -> Result<(), Error<GroupsDestroyError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        Ok(())
    } else {
        let local_var_entity: Option<GroupsDestroyError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_list(
    configuration: &configuration::Configuration,
    name: Option<&str>,
    ordering: Option<&str>,
    page: Option<i32>,
    page_size: Option<i32>,
    user: Option<&str>,
) -> Result<crate::models::PaginatedGroupList, Error<GroupsListError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/groups/", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = name {
        local_var_req_builder =
            local_var_req_builder.query(&[("name", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = ordering {
        local_var_req_builder =
            local_var_req_builder.query(&[("ordering", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = page {
        local_var_req_builder =
            local_var_req_builder.query(&[("page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = page_size {
        local_var_req_builder =
            local_var_req_builder.query(&[("page_size", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = user {
        local_var_req_builder =
            local_var_req_builder.query(&[("user", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsListError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_partial_update(
    configuration: &configuration::Configuration,
    id: &str,
    patched_group: Option<crate::models::PatchedGroup>,
) -> Result<crate::models::Group, Error<GroupsPartialUpdateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PATCH, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&patched_group);

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsPartialUpdateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Remove a user from the group.
pub fn groups_remove_create(
    configuration: &configuration::Configuration,
    id: &str,
    user: &str,
    group: crate::models::Group,
) -> Result<crate::models::Group, Error<GroupsRemoveCreateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/remove/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("user", &user.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&group);

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsRemoveCreateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_retrieve(
    configuration: &configuration::Configuration,
    id: &str,
) -> Result<crate::models::Group, Error<GroupsRetrieveError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsRetrieveError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}

/// Groups allow you to aggregate users for purposes of assigning grants more easily.
pub fn groups_update(
    configuration: &configuration::Configuration,
    id: &str,
    group: crate::models::Group,
) -> Result<crate::models::Group, Error<GroupsUpdateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/groups/{id}/",
        local_var_configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&group);

    let local_var_req = local_var_req_builder.build()?;
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(
            "URL {} {} elapsed: {:?}",
            method,
            &local_var_resp.url(),
            duration
        );
    }

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }

        serde_json::from_str(&local_var_content)
            .map_err(|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content))
    } else {
        let local_var_entity: Option<GroupsUpdateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        if local_var_configuration.rest_debug {
            println!(
                "RESP {} {}",
                &local_var_error.status, &local_var_error.content
            );
        }
        Err(Error::ResponseError(local_var_error))
    }
}
