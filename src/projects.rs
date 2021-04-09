use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use graphql_client::*;

pub struct Projects {}

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

pub trait ProjectsIntf {
    /// Resolve the `proj_name` to a String
    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<String>>;

    /// Get a complete list of projects for this organization.
    fn get_project_names(&self, org_id: Option<&str>) -> GraphQLResult<Vec<String>>;
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
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
                .projects
                .nodes)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}

impl ProjectsIntf for Projects {
    fn get_id(
        &self,
        org_id: Option<&str>,
        proj_name: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
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
                .expect("Primary organization not found")
                .project
                .map(|proj| proj.id))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    fn get_project_names(&self, org_id: Option<&str>) -> GraphQLResult<Vec<String>> {
        let projects = self.get_projects_full(org_id)?;
        let mut list = projects
            .into_iter()
            .map(|n| n.name)
            .collect::<Vec<String>>();
        list.sort();

        Ok(list)
    }
}
