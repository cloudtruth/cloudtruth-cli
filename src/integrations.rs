use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use crate::integrations::integrations_query::IntegrationsQueryViewerOrganizationIntegrationsNodesOn;
use graphql_client::*;

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
    pub parent: String,
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
}

/// The `Integrations` structure implements the `IntegrationsIntf` to get the information from
/// the GraphQL server.
pub struct Integrations {}

impl Integrations {
    pub fn new() -> Self {
        Self {}
    }
}

/// Converts the typename into a String value without the trailing `Integration`
impl ToString for IntegrationsQueryViewerOrganizationIntegrationsNodesOn {
    fn to_string(&self) -> String {
        let result = format!("{:?}", self);
        result.replace("Integration", "")
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
                .expect("Primary organization not found")
                .integrations
                .nodes;
            let mut integrations: Vec<IntegrationDetails> = Vec::new();

            for n in nodes.iter() {
                let mut parent_name = "None".to_string();
                if let Some(parent) = &n.parent {
                    parent_name = parent.name.clone();
                }
                integrations.push(IntegrationDetails {
                    fqn: n.fqn.clone(),
                    id: n.id.clone(),
                    integration_type: n.on.to_string(),
                    name: n.name.clone(),
                    parent: parent_name,
                });
            }
            Ok(integrations)
        } else {
            Err(GraphQLError::MissingDataError)
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
