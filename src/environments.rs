use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource};
use graphql_client::*;

pub struct Environments {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct CreateEnvironmentMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct DeleteEnvironmentMutation;

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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/environment_queries.graphql",
    response_derives = "Debug"
)]
pub struct UpdateEnvironmentMutation;

pub struct EnvironmentDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent: String,
    pub ancestors: Vec<String>,
}

impl Environments {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_id_details(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
    ) -> GraphQLResult<Option<EnvironmentDetails>> {
        let query =
            GetEnvironmentByNameQuery::build_query(get_environment_by_name_query::Variables {
                organization_id: org_id.map(|id| id.to_string()),
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
                .map(|env| EnvironmentDetails {
                    id: env.id,
                    name: env.name,
                    description: env.description.unwrap_or_default(),
                    parent: if let Some(p) = env.parent {
                        p.name
                    } else {
                        "".to_string()
                    },
                    ancestors: env
                        .ancestors
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<String>>(),
                }))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_id(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        if let Some(details) = self.get_id_details(org_id, env_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    fn get_environments_full(
        &self,
        org_id: Option<&str>,
    ) -> GraphQLResult<Vec<environments_query::EnvironmentsQueryViewerOrganizationEnvironmentsNodes>>
    {
        let query = EnvironmentsQuery::build_query(environments_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
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

    pub fn get_environment_details(
        &self,
        org_id: Option<&str>,
    ) -> GraphQLResult<Vec<EnvironmentDetails>> {
        let environments = self.get_environments_full(org_id)?;
        let mut env_info: Vec<EnvironmentDetails> = Vec::new();
        for e in environments {
            let ancestors = e.ancestors.iter().map(|v| v.name.clone()).collect();
            let mut parent: String = "".to_string();
            if let Some(p) = e.parent {
                parent = p.name;
            }
            env_info.push(EnvironmentDetails {
                id: e.id,
                name: e.name,
                description: e.description.unwrap_or_default(),
                ancestors,
                parent,
            })
        }
        env_info.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(env_info)
    }

    pub fn create_environment(
        &self,
        org_id: Option<&str>,
        env_name: Option<&str>,
        description: Option<&str>,
        parent_id: String,
    ) -> GraphQLResult<Option<String>> {
        let query =
            CreateEnvironmentMutation::build_query(create_environment_mutation::Variables {
                organization_id: org_id.map(|o| o.to_string()),
                environment_name: env_name.unwrap().to_string(),
                parent_id,
                description: description.map(|d| d.to_string()),
            });
        let response_body =
            graphql_request::<_, create_environment_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Environment,
                Operation::Create,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.create_environment.errors;

            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.create_environment.environment.map(|p| p.id))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn delete_environment(&self, environment_id: String) -> GraphQLResult<Option<String>> {
        let query =
            DeleteEnvironmentMutation::build_query(delete_environment_mutation::Variables {
                environment_id,
            });
        let response_body =
            graphql_request::<_, delete_environment_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Environment,
                Operation::Delete,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.delete_environment.errors;
            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.delete_environment.deleted_environment_id)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn update_environment(
        &self,
        environment_id: String,
        description: Option<&str>,
    ) -> GraphQLResult<Option<String>> {
        let query =
            UpdateEnvironmentMutation::build_query(update_environment_mutation::Variables {
                environment_id,
                description: description.map(|d| d.to_string()),
            });
        let response_body =
            graphql_request::<_, update_environment_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Environment,
                Operation::Update,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.update_environment.errors;

            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.update_environment.environment.map(|p| p.id))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
