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
pub struct GetEnvironmentByNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct EnvironmentsQuery;

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_id(&self, env_name: Option<&str>) -> GraphQLResult<Option<String>> {
        let query =
            GetEnvironmentByNameQuery::build_query(get_environment_by_name_query::Variables {
                organization_id: None,
                environment_name: env_name.map(|name| name.to_string()),
            });
        let response_body =
            graphql_request::<_, get_environment_by_name_query::ResponseData>(&query)?;

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

    pub fn is_valid_environment_name(&self, name: Option<&str>) -> GraphQLResult<bool> {
        let env = self.get_id(name)?;

        if env.is_some() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_environments_full(
        &self,
    ) -> GraphQLResult<Vec<environments_query::EnvironmentsQueryViewerOrganizationEnvironmentsNodes>>
    {
        let query = EnvironmentsQuery::build_query(environments_query::Variables {
            organization_id: None,
        });
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

    pub fn get_environment_names(&self) -> GraphQLResult<Vec<String>> {
        let environments = self.get_environments_full()?;
        let mut list = environments
            .into_iter()
            .map(|n| n.name)
            .collect::<Vec<String>>();
        list.sort();

        Ok(list)
    }
}
