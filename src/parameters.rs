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
            organization_id: None,
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

    /// Returns `Some(value)` if a value is configured for (parameter, environment) tuple or the
    /// environment's ancestry chain has a value configured at some level.
    ///
    /// Return `None` if a parameter exists but does not have a value configured for the given
    /// environment and the environment's ancestry chain does not have a value configured at any
    /// level.
    pub fn get_body(
        &self,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(env_name, key_name)?;

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

    pub fn get_id(&self, env_name: Option<&str>, key_name: &str) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(env_name, key_name)?;

        Ok(parameter.map(|p| p.id))
    }

    fn get_parameter_full(
        &self,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<
        Option<get_parameter_by_name_query::GetParameterByNameQueryViewerOrganizationParameter>,
    > {
        let query = GetParameterByNameQuery::build_query(get_parameter_by_name_query::Variables {
            organization_id: None,
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

    pub fn get_parameter_names(&self, env_id: Option<String>) -> GraphQLResult<Vec<String>> {
        let query = ParametersQuery::build_query(parameters_query::Variables {
            organization_id: None,
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
