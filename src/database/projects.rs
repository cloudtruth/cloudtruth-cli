use crate::database::{
    auth_details, response_message, OpenApiConfig, ProjectDetails, ProjectError, PAGE_SIZE,
};

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{PatchedProject, ProjectCreate};
use std::result::Result;

const NO_DESC_CONTAINS: Option<&str> = None;
const NO_NAME_CONTAINS: Option<&str> = None;
const NO_ORDERING: Option<&str> = None;

pub struct Projects {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> ProjectError {
    ProjectError::ResponseError(response_message(status, content))
}

fn auth_error(content: &str) -> ProjectError {
    ProjectError::Authentication(auth_details(content))
}

impl Projects {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the details for `proj_name`
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
    ) -> Result<Option<ProjectDetails>, ProjectError> {
        let response = projects_list(
            rest_cfg,
            NO_DESC_CONTAINS,
            Some(proj_name),
            NO_NAME_CONTAINS,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let proj = &list[0];
                        Ok(Some(ProjectDetails::from(proj)))
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }

    /// Resolve the `proj_name` to a String
    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
    ) -> Result<Option<String>, ProjectError> {
        if let Some(details) = self.get_details_by_name(rest_cfg, proj_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    /// Get a complete list of projects for this organization.
    pub fn get_project_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<ProjectDetails>, ProjectError> {
        let response = projects_list(
            rest_cfg,
            NO_DESC_CONTAINS,
            None,
            NO_NAME_CONTAINS,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    let mut details: Vec<ProjectDetails> =
                        list.iter().map(ProjectDetails::from).collect();
                    details.sort_by(|l, r| l.name.cmp(&r.name));
                    Ok(details)
                }
                None => Ok(vec![]),
            },
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }

    /// Create a project with the specified name/description
    pub fn create_project(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, ProjectError> {
        let proj = ProjectCreate {
            name: proj_name.to_string(),
            description: description.map(String::from),
            depends_on: None,
        };
        let response = projects_create(rest_cfg, proj);
        match response {
            // return the project id of the newly minted project
            Ok(project) => Ok(Some(project.id)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }

    /// Delete the specified project
    pub fn delete_project(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
    ) -> Result<Option<String>, ProjectError> {
        let response = projects_destroy(rest_cfg, project_id);
        match response {
            Ok(_) => Ok(Some(project_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }

    /// Update the specified project
    pub fn update_project(
        &self,
        rest_cfg: &OpenApiConfig,
        project_name: &str,
        project_id: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, ProjectError> {
        let proj = PatchedProject {
            url: None,
            id: None,
            name: Some(project_name.to_string()),
            description: description.map(|d| d.to_string()),
            created_at: None,
            modified_at: None,
            pushes: None,
            depends_on: None,
            dependents: None,
        };
        let response = projects_partial_update(rest_cfg, project_id, Some(proj));
        match response {
            Ok(project) => Ok(Some(project.id)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }
}
