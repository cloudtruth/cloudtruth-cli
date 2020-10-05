use crate::environments::Environments;
use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use graphql_client::*;

pub struct Parameters {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct CreateParameterQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct GetParameterByNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct ParametersQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct UpdateParameterQuery;

impl Parameters {
    pub fn new() -> Self {
        Self {}
    }

    fn find_or_create(&self, env_name: Option<&str>, key_name: &str) -> GraphQLResult<String> {
        let create_query = CreateParameterQuery::build_query(create_parameter_query::Variables {
            key_name: key_name.to_string(),
        });
        let create_response_body =
            graphql_request::<_, create_parameter_query::ResponseData>(&create_query)?;

        if let Some(errors) = create_response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = create_response_body.data {
            let create_parameter = data.create_parameter.unwrap();
            if !create_parameter.errors.is_empty() {
                // Try to fetch the parameter if we're unable to create it.
                let id = self.get_id(env_name, key_name)?;

                Ok(id.unwrap())
            } else {
                Ok(create_parameter.parameter.unwrap().id)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_body(
        &self,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(env_name, key_name)?;

        Ok(parameter.and_then(|p| p.environment_value.and_then(|ev| ev.parameter_value)))
    }

    pub fn get_id(&self, env_name: Option<&str>, key_name: &str) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(env_name, key_name)?;

        Ok(parameter.map(|p| p.id))
    }

    fn get_parameter_full(
        &self,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<Option<get_parameter_by_name_query::GetParameterByNameQueryViewerParameter>>
    {
        let query = GetParameterByNameQuery::build_query(get_parameter_by_name_query::Variables {
            env_name: env_name.map(|name| name.to_string()),
            key_name: key_name.to_string(),
        });
        let response_body =
            graphql_request::<_, get_parameter_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data.viewer.parameter)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_parameters(&self, env_id: Option<String>) -> GraphQLResult<Vec<String>> {
        let query = ParametersQuery::build_query(parameters_query::Variables {
            environment_id: env_id,
        });
        let response_body = graphql_request::<_, parameters_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .parameters
                .nodes
                .into_iter()
                .map(|n| n.key_name)
                .collect())
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn set_parameter(
        &self,
        env_name: Option<&str>,
        key_name: &str,
        value: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let environment_id = Environments::new().get_id(env_name)?;
        let parameter_id = self.find_or_create(env_name, key_name)?;

        let query = UpdateParameterQuery::build_query(update_parameter_query::Variables {
            id: parameter_id,
            environment_id,
            value: value.map(|v| v.to_string()),
        });
        let response_body = graphql_request::<_, update_parameter_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .update_parameter
                .and_then(|update| update.parameter.map(|p| p.id)))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
