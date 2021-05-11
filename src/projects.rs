use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource, NO_ORG_ERROR};
use graphql_client::*;

pub struct Projects {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/project_queries.graphql",
    response_derives = "Debug"
)]
pub struct CreateProjectMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/project_queries.graphql",
    response_derives = "Debug"
)]
pub struct DeleteProjectMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/project_queries.graphql",
    response_derives = "Debug"
)]
pub struct GetProjectByNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/project_queries.graphql",
    response_derives = "Debug"
)]
pub struct ProjectsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/project_queries.graphql",
    response_derives = "Debug"
)]
pub struct UpdateProjectMutation;

pub struct ProjectDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

pub trait ProjectsIntf {
    /// Resolve the `proj_name` to a String
    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<String>>;

    /// Get the details for `proj_name`
    fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<ProjectDetails>>;

    /// Create a project with the specified name/description
    fn create_project(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
        description: Option<&str>,
    ) -> GraphQLResult<Option<String>>;

    /// Update the specified project
    fn update_project(
        &self,
        proj_name: String,
        proj_id: String,
        description: Option<&str>,
    ) -> GraphQLResult<Option<String>>;

    /// Delete the specified project
    fn delete_project(&self, proj_id: String) -> GraphQLResult<Option<String>>;

    /// Get a complete list of projects for this organization.
    fn get_project_details(&self, org_id: Option<&str>) -> GraphQLResult<Vec<ProjectDetails>>;
}

impl Projects {
    pub fn new() -> Self {
        Self {}
    }

    fn get_projects_full(
        &self,
        org_id: Option<&str>,
    ) -> GraphQLResult<Vec<projects_query::ProjectsQueryViewerOrganizationProjectsNodes>> {
        let query = ProjectsQuery::build_query(projects_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
        });
        let response_body = graphql_request::<_, projects_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data.viewer.organization.expect(NO_ORG_ERROR).projects.nodes)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}

impl ProjectsIntf for Projects {
    fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<ProjectDetails>> {
        let query = GetProjectByNameQuery::build_query(get_project_by_name_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            project_name: proj_name.map(|name| name.to_string()),
        });
        let response_body = graphql_request::<_, get_project_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect(NO_ORG_ERROR)
                .project
                .map(|proj| ProjectDetails {
                    id: proj.id,
                    name: proj.name,
                    description: proj.description.unwrap_or_default(),
                    is_default: proj.default_for_organization,
                }))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        if let Some(details) = self.get_details_by_name(org_id, proj_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    fn get_project_details(&self, org_id: Option<&str>) -> GraphQLResult<Vec<ProjectDetails>> {
        let projects = self.get_projects_full(org_id)?;
        let mut list: Vec<ProjectDetails> = projects
            .into_iter()
            .map(|v| ProjectDetails {
                id: v.id,
                name: v.name,
                description: v.description.unwrap_or_default(),
                is_default: v.default_for_organization,
            })
            .collect();
        list.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(list)
    }

    fn create_project(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
        description: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let query = CreateProjectMutation::build_query(create_project_mutation::Variables {
            organization_id: org_id.map(|o| o.to_string()),
            project_name: proj_name.unwrap().to_string(),
            description: description.map(|d| d.to_string()),
        });
        let response_body = graphql_request::<_, create_project_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Project,
                Operation::Create,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.create_project.errors;

            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.create_project.project.map(|p| p.id))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    fn delete_project(&self, project_id: String) -> GraphQLResult<Option<String>> {
        let query =
            DeleteProjectMutation::build_query(delete_project_mutation::Variables { project_id });
        let response_body = graphql_request::<_, delete_project_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Project,
                Operation::Delete,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.delete_project.errors;
            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else if let Some(project) = data.delete_project.project {
                Ok(Some(project.id))
            } else {
                Ok(None)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    fn update_project(
        &self,
        project_name: String,
        project_id: String,
        description: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let query = UpdateProjectMutation::build_query(update_project_mutation::Variables {
            project_name,
            project_id,
            description: description.map(|d| d.to_string()),
        });
        let response_body = graphql_request::<_, update_project_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Project,
                Operation::Update,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.update_project.errors;

            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.update_project.project.map(|p| p.id))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
