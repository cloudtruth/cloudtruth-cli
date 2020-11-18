use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use graphql_client::*;

pub struct Environments {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct EnvironmentsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct GetDefaultEnvironmentQuery;

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_id(&self, env_name: Option<&str>) -> GraphQLResult<Option<String>> {
        if let Some(env_name) = env_name {
            let environments = self.get_environments_full()?;
            let env = environments.iter().find(|env| env.name == env_name);

            if let Some(env) = env {
                Ok(Some(env.id.to_string()))
            } else {
                Ok(None)
            }
        } else {
            let query = GetDefaultEnvironmentQuery::build_query(
                get_default_environment_query::Variables {},
            );
            let response_body =
                graphql_request::<_, get_default_environment_query::ResponseData>(&query)?;

            if let Some(errors) = response_body.errors {
                Err(GraphQLError::ResponseError(errors))
            } else if let Some(data) = response_body.data {
                Ok(data
                    .viewer
                    .organization
                    .expect("Primary organization not found")
                    .environment
                    .map(|env| env.id))
            } else {
                Err(GraphQLError::MissingDataError)
            }
        }
    }

    pub fn is_valid_environment_name(&self, name: Option<&str>) -> GraphQLResult<bool> {
        if let Some(name) = name {
            let environments = self.get_environments_full()?;

            Ok(environments.iter().any(|env| env.name == name))
        } else {
            // The default environment is always a valid name.
            Ok(true)
        }
    }

    fn get_environments_full(
        &self,
    ) -> GraphQLResult<Vec<environments_query::EnvironmentsQueryViewerOrganizationEnvironmentsNodes>>
    {
        let query = EnvironmentsQuery::build_query(environments_query::Variables {});
        let response_body = graphql_request::<_, environments_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
                .environments
                .nodes)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_environments(&self) -> GraphQLResult<Vec<String>> {
        let environments = self.get_environments_full()?;
        let mut list = environments
            .into_iter()
            .map(|n| n.name)
            .collect::<Vec<String>>();
        list.sort();

        Ok(list)
    }
}
