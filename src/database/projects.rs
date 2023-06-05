use crate::database::{
    auth_details, last_from_url, page_size, response_message, OpenApiConfig, ProjectDetails,
    ProjectError, NO_PAGE_COUNT, NO_PAGE_SIZE,
};

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{PatchedProjectUpdate, ProjectCopy, ProjectCreate};
use std::collections::HashMap;
use std::result::Result;

const NO_DESC_CONTAINS: Option<&str> = None;
const NO_NAME_CONTAINS: Option<&str> = None;
const NO_ORDERING: Option<&str> = None;

pub struct Projects {}

/// This is used to map from an Project's URL to the Name.
pub type ProjectUrlMap = HashMap<String, String>;

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

    /// This provides a means to get an entire list of project URLs to names.
    pub fn get_url_name_map(&self, rest_cfg: &OpenApiConfig) -> ProjectUrlMap {
        let mut result: ProjectUrlMap = ProjectUrlMap::new();
        let mut page_count = 1;
        loop {
            let response = projects_list(
                rest_cfg,
                NO_DESC_CONTAINS,
                None,
                NO_NAME_CONTAINS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
            );
            if let Ok(data) = response {
                if let Some(projects) = data.results {
                    for prj in projects {
                        result.insert(prj.url, prj.name);
                    }
                    page_count += 1;
                } else {
                    break;
                }
                if data.next.is_none() || data.next.as_ref().unwrap().is_empty() {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }

    /// Use the project URL to get the corresponding name.
    pub fn get_name_from_url(&self, rest_cfg: &OpenApiConfig, url: &str) -> String {
        let id = last_from_url(url);
        if id.is_empty() {
            "".to_owned()
        } else {
            let response = projects_retrieve(rest_cfg, id);
            if let Ok(project) = response {
                project.name
            } else {
                "".to_owned()
            }
        }
    }

    /// Get the details for `proj_name`
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        resolve_parent: bool,
    ) -> Result<Option<ProjectDetails>, ProjectError> {
        let response = projects_list(
            rest_cfg,
            NO_DESC_CONTAINS,
            Some(proj_name),
            NO_NAME_CONTAINS,
            NO_ORDERING,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
        );

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let proj = &list[0];
                        let mut details = ProjectDetails::from(proj);
                        if resolve_parent {
                            details.parent_name =
                                self.get_name_from_url(rest_cfg, &details.parent_url);
                        }
                        Ok(Some(details))
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
        if let Some(details) = self.get_details_by_name(rest_cfg, proj_name, false)? {
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
        let mut projects: Vec<ProjectDetails> = vec![];
        let mut url_map: ProjectUrlMap = ProjectUrlMap::new();
        let mut page_count = 1;
        loop {
            let response = projects_list(
                rest_cfg,
                NO_DESC_CONTAINS,
                None,
                NO_NAME_CONTAINS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
            );

            match response {
                Ok(data) => {
                    if let Some(list) = data.results {
                        for api_prj in list {
                            let details = ProjectDetails::from(&api_prj);
                            url_map.insert(details.url.clone(), details.name.clone());
                            projects.push(details);
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() || data.next.as_ref().unwrap().is_empty() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => match content.status.as_u16() {
                    401 => return Err(auth_error(&content.content)),
                    403 => return Err(auth_error(&content.content)),
                    _ => return Err(response_error(&content.status, &content.content)),
                },
                Err(e) => return Err(ProjectError::UnhandledError(e.to_string())),
            }
        }
        // populate the parent names
        for prj in &mut projects {
            if !prj.parent_url.is_empty() {
                // concurrent user updates can cause parent_url to not be present in url_map
                // handle the case by setting parent_url and parent_name to None
                if let Some(parent_name) = url_map.get(&prj.parent_url) {
                    prj.parent_name = parent_name.clone()
                } else {
                    prj.parent_name = String::new();
                    prj.parent_url = String::new()
                }
            }
        }
        projects.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(projects)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_all_descendants_from_list(
        &self,
        parent_url: &str,
        all_projects: &[ProjectDetails],
    ) -> Vec<ProjectDetails> {
        let mut descendants = vec![];
        for prj in all_projects {
            if prj.parent_url == parent_url {
                descendants.push(prj.clone());
                // recursively get the children
                let mut grandchildren = self.get_all_descendants_from_list(&prj.url, all_projects);
                descendants.append(&mut grandchildren);
            }
        }

        descendants
    }

    pub fn get_project_descendants(
        &self,
        rest_cfg: &OpenApiConfig,
        parent_name: &str,
    ) -> Result<Vec<ProjectDetails>, ProjectError> {
        let all_details = self.get_project_details(rest_cfg)?;

        // start by finding the specified parent by name
        let parent = all_details
            .iter()
            .find(|&d| d.name == parent_name)
            .expect("No project found with that name")
            .clone();
        let descendants = self.get_all_descendants_from_list(&parent.url, &all_details);
        Ok(descendants)
    }

    /// Create a project with the specified name/description
    pub fn create_project(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        description: Option<&str>,
        parent_url: Option<&str>,
        parameter_name_pattern: Option<&str>,
    ) -> Result<Option<String>, ProjectError> {
        let proj = ProjectCreate {
            name: proj_name.to_string(),
            description: description.map(String::from),
            depends_on: parent_url.map(String::from),
            parameter_name_pattern: parameter_name_pattern.map(String::from),
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
        parent_url: Option<&str>,
        parameter_name_pattern: Option<&str>,
    ) -> Result<Option<String>, ProjectError> {
        let proj = PatchedProjectUpdate {
            id: None,
            name: Some(project_name.to_string()),
            description: description.map(|d| d.to_string()),
            created_at: None,
            modified_at: None,
            depends_on: parent_url.map(String::from),
            access_controlled: None,
            role: None,
            parameter_name_pattern: parameter_name_pattern.map(String::from),
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

    pub fn copy_project(
        &self,
        rest_cfg: &OpenApiConfig,
        src_project_id: &str,
        name: &str,
        description: Option<&str>,
        recursive: bool,
        child_names: Option<HashMap<String, String>>,
    ) -> Result<String, ProjectError> {
        let response = projects_copy_create(
            rest_cfg,
            src_project_id,
            ProjectCopy {
                name: name.to_owned(),
                description: description.map(String::from),
                child_project_names: child_names,
                recursive: Some(recursive),
                depends_on: None,
            },
        );
        match response {
            Ok(proj) => Ok(proj.id),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(ProjectError::UnhandledError(e.to_string())),
        }
    }
}
