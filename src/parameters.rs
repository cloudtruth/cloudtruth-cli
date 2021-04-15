use crate::config::DEFAULT_PROJ_NAME;
use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource};
use graphql_client::*;
use std::collections::HashMap;

pub struct Parameters {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct DeleteParameterMutation;

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
pub struct ParametersDetailQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/parameter_queries.graphql",
    response_derives = "Debug"
)]
pub struct UpsertParameterMutation;

pub struct ParameterDetails {
    pub id: String,
    pub key: String,
    pub value: String,
    pub secret: bool,
    pub description: String,
    pub source: String,
}

impl Parameters {
    pub fn new() -> Self {
        Self {}
    }

    /// Deletes the specified parameter by ID
    ///
    /// On success, it returns the deleted parameter ID. On failure, it returns a GraphQLError.
    fn delete_param_by_id(&self, param_id: String) -> GraphQLResult<Option<String>> {
        let query = DeleteParameterMutation::build_query(delete_parameter_mutation::Variables {
            parameter_id: param_id,
        });
        let response_body = graphql_request::<_, delete_parameter_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Parameter,
                Operation::Delete,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.delete_parameter.errors;
            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.delete_parameter.deleted_parameter_id)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    /// Deletes the specified parameter from the specified project/environment.
    ///
    /// On success, it returns the ID of the deleted value. On failure, it returns a GraphQlError
    /// with more failure information.
    pub fn delete_parameter(
        &self,
        org_id: Option<&str>,
        proj_name: Option<String>,
        env_name: Option<&str>,
        key_name: &str,
    ) -> GraphQLResult<Option<String>> {
        // The only delete mechanism is by parameter ID, so start by querying the parameter info.
        let parameter = self.get_parameter_full(org_id, env_name, proj_name, key_name)?;

        if let Some(parameter) = parameter {
            self.delete_param_by_id(parameter.id)
        } else {
            Err(GraphQLError::ParameterNotFoundError(key_name.to_string()))
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
        org_id: Option<&str>,
        env_name: Option<&str>,
        proj_name: Option<String>,
        key_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let parameter = self.get_parameter_full(org_id, env_name, proj_name, key_name)?;

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

    pub fn get_parameter_full(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
        proj_name: Option<String>,
        key_name: &str,
    ) -> GraphQLResult<
        Option<
            get_parameter_by_name_query::GetParameterByNameQueryViewerOrganizationProjectParameter,
        >,
    > {
        let query = GetParameterByNameQuery::build_query(get_parameter_by_name_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            env_name: env_name.map(|name| name.to_string()),
            project_name: proj_name.clone(),
            key_name: key_name.to_string(),
        });
        let response_body =
            graphql_request::<_, get_parameter_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                Ok(project.parameter)
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    proj_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_parameter_names(
        &self,
        org_id: Option<&str>,
        env_id: Option<String>,
        proj_name: Option<String>,
    ) -> GraphQLResult<Vec<String>> {
        let query = ParametersQuery::build_query(parameters_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            environment_id: env_id,
            project_name: proj_name.clone(),
        });
        let response_body = graphql_request::<_, parameters_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                Ok(project
                    .parameters
                    .nodes
                    .into_iter()
                    .map(|n| n.key_name)
                    .collect())
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    proj_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_parameter_values(
        &self,
        org_id: Option<&str>,
        env_id: Option<String>,
        proj_name: Option<String>,
    ) -> GraphQLResult<HashMap<String, String>> {
        let query = ParametersQuery::build_query(parameters_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            environment_id: env_id,
            project_name: proj_name.clone(),
        });
        let response_body = graphql_request::<_, parameters_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            let mut env_vars = HashMap::new();
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                for p in project.parameters.nodes {
                    if let Some(env_value) = p.environment_value {
                        if let Some(param_value) = env_value.parameter_value {
                            env_vars.insert(p.key_name, param_value);
                        }
                    }
                }
                Ok(env_vars)
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    proj_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_parameter_details(
        &self,
        org_id: Option<&str>,
        env_id: Option<String>,
        proj_name: Option<String>,
    ) -> GraphQLResult<Vec<ParameterDetails>> {
        let query = ParametersDetailQuery::build_query(parameters_detail_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
            environment_id: env_id,
            project_name: proj_name.clone(),
        });
        let response_body = graphql_request::<_, parameters_detail_query::ResponseData>(&query)?;
        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                let mut env_vars: Vec<ParameterDetails> = Vec::new();
                for p in project.parameters.nodes {
                    let mut param_value: String = "".to_string();
                    let mut source: String = "".to_string();

                    if let Some(env_value) = p.environment_value {
                        if let Some(inherit) = env_value.inherited_from {
                            source = inherit.name;
                        } else {
                            source = env_value.environment.name;
                        }
                        param_value = env_value.parameter_value.unwrap_or_default();
                    }

                    // Add an entry for every parameter, even if it has no value or source
                    env_vars.push(ParameterDetails {
                        id: p.id,
                        key: p.key_name,
                        value: param_value,
                        secret: p.is_secret,
                        description: p.description.unwrap_or_default(),
                        source,
                    });
                }
                Ok(env_vars)
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    proj_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn set_parameter(
        &self,
        proj_id: Option<String>,
        env_name: Option<&str>,
        key_name: &str,
        value: Option<&str>,
        description: Option<&str>,
        secret: Option<bool>,
    ) -> GraphQLResult<Option<String>> {
        let query = UpsertParameterMutation::build_query(upsert_parameter_mutation::Variables {
            project_id: proj_id,
            environment_name: env_name.map(|env| env.to_string()),
            key_name: key_name.to_string(),
            value: value.map(|v| v.to_string()),
            description: description.map(|v| v.to_string()),
            secret,
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
