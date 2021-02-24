use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource};
use graphql_client::*;

pub struct Parameters {}

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
pub struct UpsertParameterMutation;

impl Parameters {
    pub fn new() -> Self {
        Self {}
    }

    /// Returns `Some(value)` if a value is configured for (parameter, environment) tuple or the
    /// environment's ancestry chain has a value configured at some level.
    ///
    /// Return `None` if a parameter exists but does not have a value configured for the given
    /// environment and the environment's ancestry chain does not have a value configured at any
    /// level.
    pub fn get_body(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(org_id, env_name, key_name)?;

        // The query response can take multiple shapes depending on the state of the CloudTruth
        // parameter store.
        //
        // 1) parameter = None -> Parameter does not exist
        // 2) parameter.environment_value = None -> Environment does not exist
        // 3) parameter.environment_value.parameter_value = None -> The parameter and the
        // environment both exist, but the (parameter, environment) combination either does not have
        // a configured value or inherits from an environment that does not have a configured value
        // 4) parameter.environment_value.parameter_value = Some -> The parameter and the
        // environment both exist and there is a value set or inherited in the (parameter, environment)
        // combination.

        if let Some(parameter) = parameter {
            if let Some(environment_value) = parameter.environment_value {
                Ok(environment_value.parameter_value)
            } else {
                Err(GraphQLError::EnvironmentNotFoundError(
                    env_name
                        .expect("The default environment should always be found")
                        .to_string(),
                ))
            }
        } else {
            Err(GraphQLError::ParameterNotFoundError(key_name.to_string()))
        }
    }

    fn get_parameter_full(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<
        Option<get_parameter_by_name_query::GetParameterByNameQueryViewerOrganizationParameter>,
    > {
        let query = GetParameterByNameQuery::build_query(get_parameter_by_name_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            env_name: env_name.map(|name| name.to_string()),
            key_name: key_name.to_string(),
        });
        let response_body =
            graphql_request::<_, get_parameter_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
                .parameter)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_parameter_names(
        &self,
        org_id: Option<&str>,
        env_id: Option<String>,
    ) -> GraphQLResult<Vec<String>> {
        let query = ParametersQuery::build_query(parameters_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            environment_id: env_id,
        });
        let response_body = graphql_request::<_, parameters_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
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
        org_id: Option<&str>,
        env_name: Option<&str>,
        key_name: &str,
        value: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let query = UpsertParameterMutation::build_query(upsert_parameter_mutation::Variables {
            org_id: org_id.map(|id| id.to_string()),
            environment_name: env_name.map(|env| env.to_string()),
            key_name: key_name.to_string(),
            value: value.map(|v| v.to_string()),
        });
        let response_body = graphql_request::<_, upsert_parameter_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Parameter,
                Operation::Upsert,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.upsert_parameter.errors;

            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.upsert_parameter.parameter.map(|p| p.id))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
