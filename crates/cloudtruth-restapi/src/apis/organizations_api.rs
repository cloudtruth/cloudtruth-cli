/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

use reqwest;
use std::time::Instant;

use super::{configuration, Error};
use crate::apis::{handle_serde_error, ResponseContent};

/// struct for typed errors of method [`organizations_create`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsCreateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`organizations_destroy`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsDestroyError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`organizations_list`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsListError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`organizations_partial_update`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsPartialUpdateError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`organizations_retrieve`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsRetrieveError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`organizations_update`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrganizationsUpdateError {
    UnknownValue(serde_json::Value),
}

pub fn organizations_create(
    configuration: &configuration::Configuration,
    organization_create: crate::models::OrganizationCreate,
) -> Result<crate::models::Organization, Error<OrganizationsCreateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/",
        local_var_configuration.base_path
    );
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
    local_var_req_builder = local_var_req_builder.json(&organization_create);

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
        let local_var_entity: Option<OrganizationsCreateError> =
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

pub fn organizations_destroy(
    configuration: &configuration::Configuration,
    id: &str,
) -> Result<(), Error<OrganizationsDestroyError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/{id}/",
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
        let local_var_entity: Option<OrganizationsDestroyError> =
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

pub fn organizations_list(
    configuration: &configuration::Configuration,
    name: Option<&str>,
    ordering: Option<&str>,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<crate::models::PaginatedOrganizationList, Error<OrganizationsListError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/",
        local_var_configuration.base_path
    );
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
        let local_var_entity: Option<OrganizationsListError> =
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

pub fn organizations_partial_update(
    configuration: &configuration::Configuration,
    id: &str,
    patched_organization: Option<crate::models::PatchedOrganization>,
) -> Result<crate::models::Organization, Error<OrganizationsPartialUpdateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/{id}/",
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
    local_var_req_builder = local_var_req_builder.json(&patched_organization);

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
        let local_var_entity: Option<OrganizationsPartialUpdateError> =
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

pub fn organizations_retrieve(
    configuration: &configuration::Configuration,
    id: &str,
) -> Result<crate::models::Organization, Error<OrganizationsRetrieveError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/{id}/",
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
        let local_var_entity: Option<OrganizationsRetrieveError> =
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

pub fn organizations_update(
    configuration: &configuration::Configuration,
    id: &str,
    organization: crate::models::Organization,
) -> Result<crate::models::Organization, Error<OrganizationsUpdateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v1/organizations/{id}/",
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
    local_var_req_builder = local_var_req_builder.json(&organization);

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
        let local_var_entity: Option<OrganizationsUpdateError> =
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
