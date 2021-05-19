use crate::config::{DEFAULT_ENV_NAME, DEFAULT_PROJ_NAME};
use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource, NO_ORG_ERROR};
use crate::parameters::export_parameters_query::ExportParametersFormatEnum;
use graphql_client::*;
use std::collections::HashMap;
use std::str::FromStr;

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
pub struct ExportParametersQuery;

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

#[derive(Debug)]
pub struct ParameterDetails {
    pub id: String,
    pub key: String,
    pub value: String,
    pub secret: bool,
    pub description: String,
    pub source: String,
    pub dynamic: bool,
    pub fqn: String,
    pub jmes_path: String,
}

/// Converts to ExportParametersFormatEnum from a &str.
impl FromStr for ExportParametersFormatEnum {
    type Err = ();

    fn from_str(input: &str) -> Result<ExportParametersFormatEnum, Self::Err> {
        match input {
            "docker" => Ok(ExportParametersFormatEnum::DOCKER),
            "dotenv" => Ok(ExportParametersFormatEnum::DOTENV),
            "shell" => Ok(ExportParametersFormatEnum::SHELL),
            _ => Err(()),
        }
    }
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
        let parameter = self.get_details_by_name(org_id, env_name, proj_name, key_name)?;

        if let Some(parameter) = parameter {
            self.delete_param_by_id(parameter.id)
        } else {
            Err(GraphQLError::ParameterNotFoundError(key_name.to_string()))
        }
    }

    /// Exports the specified parameters and values to a well-known output type.
    ///
    /// On success, returns a formatted string containing the specified parameters/values in
    /// the specified output format.
    pub fn export_parameters(
        &self,
        organization_id: Option<&str>,
        project_name: Option<String>,
        environment_name: Option<&str>,
        options: export_parameters_query::ExportParametersOptions,
        format: ExportParametersFormatEnum,
    ) -> GraphQLResult<Option<String>> {
        let query = ExportParametersQuery::build_query(export_parameters_query::Variables {
            organization_id: organization_id.map(|id| id.to_string()),
            project_name: project_name.clone(),
            environment_name: environment_name.map(|name| name.to_string()),
            format,
            options,
        });
        let response_body = graphql_request::<_, export_parameters_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data.viewer.organization.expect(NO_ORG_ERROR).project {
                Ok(project.export_parameters.and_then(|v| v.evaluated))
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    project_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    /// Fetches the `ParameterDetails` for the specified project/environment/key_name.
    ///
    /// It will return `None` if the parameter does not exist. Other errors will be returned
    /// if project/environments are not found.
    pub fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
        proj_name: Option<String>,
        key_name: &str,
    ) -> GraphQLResult<Option<ParameterDetails>> {
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
            if let Some(project) = data.viewer.organization.expect(NO_ORG_ERROR).project {
                if let Some(param) = project.parameter {
                    if let Some(env_value) = param.environment_value {
                        let source: String;
                        let mut fqn = "".to_string();
                        if let Some(inherit) = env_value.inherited_from {
                            source = inherit.name;
                        } else {
                            source = env_value.environment.name;
                        }
                        if let Some(file) = env_value.integration_file {
                            fqn = file.fqn;
                        }
                        let param_value = env_value.parameter_value.unwrap_or_default();
                        let jmes_path = env_value.jmes_path.unwrap_or_default();
                        Ok(Some(ParameterDetails {
                            id: param.id.clone(),
                            key: param.key_name.clone(),
                            value: param_value,
                            secret: param.is_secret,
                            description: param.description.unwrap_or_default(),
                            source,
                            dynamic: param.has_dynamic_value,
                            fqn,
                            jmes_path,
                        }))
                    } else {
                        let name = env_name.unwrap_or(DEFAULT_ENV_NAME).to_string();
                        Err(GraphQLError::EnvironmentNotFoundError(name))
                    }
                } else {
                    Err(GraphQLError::ParameterNotFoundError(key_name.to_string()))
                }
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
            if let Some(project) = data.viewer.organization.expect(NO_ORG_ERROR).project {
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
            if let Some(project) = data.viewer.organization.expect(NO_ORG_ERROR).project {
                let mut env_vars: Vec<ParameterDetails> = Vec::new();
                for p in project.parameters.nodes {
                    let mut fqn = "".to_string();
                    let mut jmes_path = "".to_string();
                    let mut param_value: String = "".to_string();
                    let mut source: String = "".to_string();

                    if let Some(env_value) = p.environment_value {
                        if let Some(inherit) = env_value.inherited_from {
                            source = inherit.name;
                        } else {
                            source = env_value.environment.name;
                        }
                        if let Some(file) = env_value.integration_file {
                            fqn = file.fqn;
                        }
                        param_value = env_value.parameter_value.unwrap_or_default();
                        jmes_path = env_value.jmes_path.unwrap_or_default();
                    }

                    // Add an entry for every parameter, even if it has no value or source
                    env_vars.push(ParameterDetails {
                        id: p.id,
                        key: p.key_name,
                        value: param_value,
                        secret: p.is_secret,
                        description: p.description.unwrap_or_default(),
                        source,
                        dynamic: p.has_dynamic_value,
                        jmes_path,
                        fqn,
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

    #[allow(clippy::too_many_arguments)]
    pub fn set_parameter(
        &self,
        proj_id: Option<String>,
        env_name: Option<&str>,
        key_name: &str,
        value: Option<&str>,
        description: Option<&str>,
        secret: Option<bool>,
        fqn: Option<&str>,
        jmes_path: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let query = UpsertParameterMutation::build_query(upsert_parameter_mutation::Variables {
            project_id: proj_id,
            environment_name: env_name.map(|env| env.to_string()),
            key_name: key_name.to_string(),
            value: value.map(|v| v.to_string()),
            description: description.map(|v| v.to_string()),
            secret,
            fqn: fqn.map(|f| f.to_string()),
            jmes_path: jmes_path.map(|j| j.to_string()),
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
