use crate::openapi::open_api_config;

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::{PatchedProject, Project, ProjectCreate};
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
        let description = api_proj.description.clone();
        ProjectDetails {
            id: api_proj.id.clone(),
            name: api_proj.name.clone(),
            description: description.unwrap_or_else(|| "".to_string()),
        }
    }
}

pub trait ProjectsIntf {
    /// Resolve the `proj_name` to a String
    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsListError>>;

    /// Get the details for `proj_name`
    fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> Result<Option<ProjectDetails>, Error<ProjectsListError>>;

    /// Create a project with the specified name/description
    fn create_project(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsCreateError>>;

    /// Update the specified project
    fn update_project(
        &self,
        proj_name: String,
        proj_id: String,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsPartialUpdateError>>;

    /// Delete the specified project
    fn delete_project(
        &self,
        proj_id: String,
    ) -> Result<Option<String>, Error<ProjectsDestroyError>>;

    /// Get a complete list of projects for this organization.
    fn get_project_details(
        &self,
        org_id: Option<&str>,
    ) -> Result<Vec<ProjectDetails>, Error<ProjectsListError>>;
}

impl Projects {
    pub fn new() -> Self {
        Self {}
    }
}

impl ProjectsIntf for Projects {
    fn get_details_by_name(
        &self,
        _org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> Result<Option<ProjectDetails>, Error<ProjectsListError>> {
        let rest_cfg = open_api_config();
        let response = projects_list(&rest_cfg, proj_name, None)?;

        if let Some(projects) = response.results {
            // TODO: handle more than one??
            let proj = &projects[0];
            Ok(Some(ProjectDetails::from(proj)))
        } else {
            Ok(None)
        }
    }

    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsListError>> {
        if let Some(details) = self.get_details_by_name(org_id, proj_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    fn get_project_details(
        &self,
        _org_id: Option<&str>,
    ) -> Result<Vec<ProjectDetails>, Error<ProjectsListError>> {
        let rest_cfg = open_api_config();
        let response = projects_list(&rest_cfg, None, None)?;

        if let Some(projects) = response.results {
            // TODO: handle more than one??
            let mut list: Vec<ProjectDetails> = projects
                .into_iter()
                .map(|v| ProjectDetails::from(&v))
                .collect();
            list.sort_by(|l, r| l.name.cmp(&r.name));
            Ok(list)
        } else {
            Ok(vec![])
        }
    }

    fn create_project(
        &self,
        _org_id: Option<&str>,
        proj_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsCreateError>> {
        let rest_cfg = open_api_config();
        let proj = ProjectCreate {
            name: proj_name.unwrap().to_string(),
            description: description.map(String::from),
        };
        let response = projects_create(&rest_cfg, proj)?;
        // return the project id of the newly minted project
        Ok(Some(response.id))
    }

    fn delete_project(
        &self,
        project_id: String,
    ) -> Result<Option<String>, Error<ProjectsDestroyError>> {
        let rest_cfg = open_api_config();
        projects_destroy(&rest_cfg, project_id.as_str())?;
        Ok(Some(project_id))
    }

    fn update_project(
        &self,
        project_name: String,
        project_id: String,
        description: Option<&str>,
    ) -> Result<Option<String>, Error<ProjectsPartialUpdateError>> {
        // TODO: allow setting more details?
        let rest_cfg = open_api_config();
        let proj = PatchedProject {
            url: None,
            id: None,
            name: Some(project_name),
            description: description.map(|d| d.to_string()),
        };
        let response = projects_partial_update(&rest_cfg, project_id.as_str(), Some(proj))?;
        Ok(Some(response.id))
    }
}
