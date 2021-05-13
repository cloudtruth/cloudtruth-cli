use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, Operation, Resource, NO_ORG_ERROR};
use crate::integrations::integrations_query::IntegrationsQueryViewerOrganizationIntegrationsNodesOn;
use graphql_client::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/integration_queries.graphql",
    response_derives = "Debug"
)]
pub struct DeleteAwsIntegrationMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/integration_queries.graphql",
    response_derives = "Debug"
)]
pub struct DeleteGithubIntegrationMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/integration_queries.graphql",
    response_derives = "Debug"
)]
pub struct IntegrationsQuery;

/// This holds the common information for Integrations, including an `integration_type` that
/// indicates the provider.
pub struct IntegrationDetails {
    pub fqn: String,
    pub id: String,
    pub integration_type: String,
    pub name: String,
}

/// This is the interface that is implemented to retrieve integration information.
///
/// This layer of abstraction is done to allow for mocking in unittest, and to potentially allow
/// for future implementations.
pub trait IntegrationsIntf {
    /// Gets a list of `IntegrationDetails` for all integration types.
    fn get_integration_details(
        &self,
        org_id: Option<&str>,
    ) -> GraphQLResult<Vec<IntegrationDetails>>;

    /// Gets a single `IntegrationDetails` object for the specified name.
    fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        int_name: Option<&str>,
        int_type: Option<&str>,
    ) -> GraphQLResult<Option<IntegrationDetails>>;

    /// Deletes the integration by ID.
    fn delete_integration(
        &self,
        integration_id: String,
        integration_type: String,
    ) -> GraphQLResult<Option<String>>;
}

/// Converts the typename into a String value without the trailing `Integration`
impl ToString for IntegrationsQueryViewerOrganizationIntegrationsNodesOn {
    fn to_string(&self) -> String {
        let result = format!("{:?}", self);
        result.replace("Integration", "")
    }
}

/// The `Integrations` structure implements the `IntegrationsIntf` to get the information from
/// the GraphQL server.
pub struct Integrations {}

impl Integrations {
    pub fn new() -> Self {
        Self {}
    }

    pub fn delete_aws_integration(&self, integration_id: String) -> GraphQLResult<Option<String>> {
        let query =
            DeleteAwsIntegrationMutation::build_query(delete_aws_integration_mutation::Variables {
                integration_id,
            });
        let response_body =
            graphql_request::<_, delete_aws_integration_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Integration,
                Operation::Delete,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.delete_aws_integration.errors;
            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.delete_aws_integration.deleted_aws_integration_id)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn delete_github_integration(
        &self,
        integration_id: String,
    ) -> GraphQLResult<Option<String>> {
        let query = DeleteGithubIntegrationMutation::build_query(
            delete_github_integration_mutation::Variables { integration_id },
        );
        let response_body =
            graphql_request::<_, delete_github_integration_mutation::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::build_query_error(
                errors,
                Resource::Integration,
                Operation::Delete,
            ))
        } else if let Some(data) = response_body.data {
            let logical_errors = data.delete_github_integration.errors;
            if !logical_errors.is_empty() {
                Err(GraphQLError::build_logical_error(to_user_errors!(
                    logical_errors
                )))
            } else {
                Ok(data.delete_github_integration.deleted_github_integration_id)
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}

impl IntegrationsIntf for Integrations {
    fn get_integration_details(
        &self,
        org_id: Option<&str>,
    ) -> GraphQLResult<Vec<IntegrationDetails>> {
        let query = IntegrationsQuery::build_query(integrations_query::Variables {
            organization_id: org_id.map(|id| id.to_string()),
        });
        let response_body = graphql_request::<_, integrations_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            let nodes = data
                .viewer
                .organization
                .expect(NO_ORG_ERROR)
                .integrations
                .nodes;
            let mut integrations: Vec<IntegrationDetails> = Vec::new();

            for n in nodes.iter() {
                integrations.push(IntegrationDetails {
                    fqn: n.fqn.clone(),
                    id: n.id.clone(),
                    integration_type: n.on.to_string(),
                    name: n.name.clone(),
                });
            }
            Ok(integrations)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    fn get_details_by_name(
        &self,
        org_id: Option<&str>,
        int_name: Option<&str>,
        int_type: Option<&str>,
    ) -> GraphQLResult<Option<IntegrationDetails>> {
        // TODO: Rick Porter 5/21 - change to use a query for a single element
        // this is an interim query that gets the whole list, and does filtering on the CLI
        let all_entries = self.get_integration_details(org_id)?;
        let mut found: Option<IntegrationDetails> = None;
        let mut integration_types: Vec<String> = Vec::new();

        // walk the list of integration details looking for matches (and duplicates)
        for entry in all_entries {
            if int_name.unwrap() != entry.name.as_str() {
                continue;
            }

            if int_type.is_none() {
                integration_types.push(entry.integration_type.clone());
                found = Some(entry);
            } else if int_type == Some(entry.integration_type.as_str()) {
                found = Some(entry);
                break;
            }
        }

        if integration_types.len() > 1 {
            let multiples = integration_types.join(", ");
            Err(GraphQLError::AmbiguousIntegrationError(
                int_name.unwrap().to_string(),
                multiples,
            ))
        } else {
            Ok(found)
        }
    }

    fn delete_integration(
        &self,
        integration_id: String,
        integration_type: String,
    ) -> GraphQLResult<Option<String>> {
        if integration_type.to_lowercase() == "aws" {
            self.delete_aws_integration(integration_id)
        } else if integration_type.to_lowercase() == "github" {
            self.delete_github_integration(integration_id)
        } else {
            Err(GraphQLError::IntegrationTypeError(integration_type))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn integration_type_to_string() {
        let mut value = IntegrationsQueryViewerOrganizationIntegrationsNodesOn::AwsIntegration;
        assert_eq!("Aws".to_string(), format!("{}", value.to_string()));
        value = IntegrationsQueryViewerOrganizationIntegrationsNodesOn::GithubIntegration;
        assert_eq!("Github".to_string(), format!("{}", value.to_string()));
    }
}
