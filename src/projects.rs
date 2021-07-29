use crate::openapi::{extract_details, OpenApiConfig, PAGE_SIZE};

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::{self, ResponseError};
use cloudtruth_restapi::models::{PatchedProject, Project, ProjectCreate};
use std::error;
use std::fmt::{self, Formatter};
use std::result::Result;

pub struct Projects {}

#[derive(Debug)]
pub struct ProjectDetails {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Converts from the OpenApi `Project` model to the CloudTruth `ProjectDetails`
impl From<&Project> for ProjectDetails {
    fn from(api_proj: &Project) -> Self {
        ProjectDetails {
            id: api_proj.id.clone(),
            name: api_proj.name.clone(),
            description: api_proj.description.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub enum ProjectError {
    AuthError(String),
    ListError(Error<ProjectsListError>),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::AuthError(msg) => write!(f, "Not Authenticated: {}", msg),
            ProjectError::ListError(e) => write!(f, "{}", e.to_string()),
        }
    }
}

impl error::Error for ProjectError {}

impl Projects {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the details for `proj_name`
    pub fn get_details_by_name(
        &self,
        rest_cfg: &mut OpenApiConfig,
        proj_name: &str,
    ) -> Result<Option<ProjectDetails>, ProjectError> {
        let response = projects_list(rest_cfg, Some(proj_name), None, PAGE_SIZE);

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
                401 => Err(ProjectError::AuthError(extract_details(&content.content))),
                403 => Err(ProjectError::AuthError(extract_details(&content.content))),
                _ => Err(ProjectError::ListError(response.unwrap_err())),
            },
            Err(e) => Err(ProjectError::ListError(e)),
        }
    }

    /// Resolve the `proj_name` to a String
    pub fn get_id(
        &self,
        rest_cfg: &mut OpenApiConfig,
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
        rest_cfg: &mut OpenApiConfig,
    ) -> Result<Vec<ProjectDetails>, ProjectError> {
        let response = projects_list(rest_cfg, None, None, PAGE_SIZE);

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
                401 => Err(ProjectError::AuthError(extract_details(&content.content))),
                403 => Err(ProjectError::AuthError(extract_details(&content.content))),
                _ => Err(ProjectError::ListError(response.unwrap_err())),
            },
            Err(e) => Err(ProjectError::ListError(e)),
        }
    }

    /// Create a project with the specified name/description
    pub fn create_project(
        &self,
        rest_cfg: &mut OpenApiConfig,
        proj_name: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsCreateError>> {
        let proj = ProjectCreate {
            name: proj_name.to_string(),
            description: description.map(String::from),
        };
        let response = projects_create(rest_cfg, proj)?;
        // return the project id of the newly minted project
        Ok(Some(response.id))
    }

    /// Delete the specified project
    pub fn delete_project(
        &self,
        rest_cfg: &mut OpenApiConfig,
        project_id: &str,
    ) -> Result<Option<String>, Error<ProjectsDestroyError>> {
        projects_destroy(rest_cfg, project_id)?;
        Ok(Some(project_id.to_string()))
    }

    /// Update the specified project
    pub fn update_project(
        &self,
        rest_cfg: &mut OpenApiConfig,
        project_name: &str,
        project_id: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsPartialUpdateError>> {
        let proj = PatchedProject {
            url: None,
            id: None,
            name: Some(project_name.to_string()),
            description: description.map(|d| d.to_string()),
            created_at: None,
            modified_at: None,
        };
        let response = projects_partial_update(rest_cfg, project_id, Some(proj))?;
        Ok(Some(response.id))
    }
}
