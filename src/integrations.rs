extern crate serde_json;
use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult, NO_ORG_ERROR};
use crate::integrations::integration_node_query::*;
use crate::integrations::integrations_query::IntegrationsQueryViewerOrganizationIntegrationsNodesOn;
use crate::integrations::IntegrationNodeQueryNodeOn::AwsIntegration;
use crate::integrations::IntegrationNodeQueryNodeOn::GithubIntegration;
use crate::integrations::IntegrationNodeQueryNodeOn::IntegrationFile;
use crate::integrations::IntegrationNodeQueryNodeOn::IntegrationFileTree;
use crate::integrations::IntegrationNodeQueryNodeOn::IntegrationServiceTree;
use graphql_client::*;
use serde_json::value::Value::Object;
use serde_json::Value;
use std::fmt::Debug;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/integration_queries.graphql",
    response_derives = "Debug"
)]
pub struct IntegrationsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/integration_queries.graphql",
    response_derives = "Debug"
)]
pub struct IntegrationNodeQuery;

/// This holds the common information for Integrations.
#[derive(Debug)]
pub struct IntegrationDetails {
    pub fqn: String,
    pub id: String,
    pub integration_type: String,
    pub name: String,
}

/// Provides basic information about the integration entries
#[derive(Debug)]
pub struct IntegrationEntry {
    pub fqn: String,
    pub id: String,
    pub name: String,
}

/// Provides information about the current node, and its' children
#[derive(Debug)]
pub struct IntegrationNode {
    pub fqn: String,
    pub id: String,
    pub name: String,
    /// `entries` are a list of "children" in the integration.
    pub entries: Vec<IntegrationEntry>,
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

    /// Get the integration node by ID
    fn get_integration_node(&self, entry_id: String) -> GraphQLResult<Option<IntegrationNode>>;
}

/// Converts the typename into a String value without the trailing `Integration`
impl ToString for IntegrationsQueryViewerOrganizationIntegrationsNodesOn {
    fn to_string(&self) -> String {
        let result = format!("{:?}", self);
        result.replace("Integration", "")
    }
}

impl From<&IntegrationNodeQueryNodeOnAwsIntegration> for IntegrationNode {
    fn from(node: &IntegrationNodeQueryNodeOnAwsIntegration) -> Self {
        let mut entries: Vec<IntegrationEntry> = Vec::new();
        for e in node.entries.iter() {
            entries.push(IntegrationEntry::from(e));
        }
        IntegrationNode {
            fqn: node.fqn.clone(),
            id: node.id.clone(),
            name: node.name.clone(),
            entries,
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnGithubIntegration> for IntegrationNode {
    fn from(node: &IntegrationNodeQueryNodeOnGithubIntegration) -> Self {
        let mut entries: Vec<IntegrationEntry> = Vec::new();
        for e in node.entries.iter() {
            entries.push(IntegrationEntry::from(e));
        }
        IntegrationNode {
            fqn: node.fqn.clone(),
            id: node.id.clone(),
            name: node.name.clone(),
            entries,
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnIntegrationServiceTree> for IntegrationNode {
    fn from(node: &IntegrationNodeQueryNodeOnIntegrationServiceTree) -> Self {
        let mut entries: Vec<IntegrationEntry> = Vec::new();
        for e in node.entries.iter() {
            entries.push(IntegrationEntry::from(e));
        }
        IntegrationNode {
            fqn: node.fqn.clone(),
            id: node.id.clone(),
            name: node.name.clone(),
            entries,
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnIntegrationFileTree> for IntegrationNode {
    fn from(node: &IntegrationNodeQueryNodeOnIntegrationFileTree) -> Self {
        let mut entries: Vec<IntegrationEntry> = Vec::new();
        for e in node.entries.iter() {
            entries.push(IntegrationEntry::from(e));
        }
        IntegrationNode {
            fqn: node.fqn.clone(),
            id: node.id.clone(),
            name: node.name.clone(),
            entries,
        }
    }
}

/// Walks a Json Value and (recursively) creates a list of keys using "dot" notation.
///
/// As an example:
///    { "a": ["an", "array"], "b": { "an": "object" } }
/// turns into [ a, b.an }
fn get_keys(v: Value) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    if let Object(map) = v {
        for (key, value) in map {
            if !value.is_object() {
                keys.push(key);
            } else {
                let sub_keys = get_keys(value);
                for sk in sub_keys {
                    keys.push(format!("{}.{}", key, sk));
                }
            }
        }
    }
    keys
}

impl From<&IntegrationNodeQueryNodeOnIntegrationFile> for IntegrationNode {
    fn from(node: &IntegrationNodeQueryNodeOnIntegrationFile) -> Self {
        let mut entries: Vec<IntegrationEntry> = Vec::new();
        if let Some(json) = &node.json {
            if let Ok(v) = serde_json::from_str::<Value>(&json.as_str()) {
                let key_names = get_keys(v);
                for k in key_names {
                    entries.push(IntegrationEntry {
                        fqn: node.fqn.clone(),
                        id: "".to_string(),
                        name: format!("{{{{ {} }}}}", k),
                    })
                }
            }
        }
        IntegrationNode {
            fqn: node.fqn.clone(),
            id: node.id.clone(),
            name: node.name.clone(),
            entries,
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnAwsIntegrationEntries> for IntegrationEntry {
    fn from(entry: &IntegrationNodeQueryNodeOnAwsIntegrationEntries) -> Self {
        IntegrationEntry {
            fqn: entry.fqn.clone(),
            id: entry.id.clone(),
            name: entry.name.clone(),
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnGithubIntegrationEntries> for IntegrationEntry {
    fn from(entry: &IntegrationNodeQueryNodeOnGithubIntegrationEntries) -> Self {
        IntegrationEntry {
            fqn: entry.fqn.clone(),
            id: entry.id.clone(),
            name: entry.name.clone(),
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnIntegrationServiceTreeEntries> for IntegrationEntry {
    fn from(entry: &IntegrationNodeQueryNodeOnIntegrationServiceTreeEntries) -> Self {
        IntegrationEntry {
            fqn: entry.fqn.clone(),
            id: entry.id.clone(),
            name: entry.name.clone(),
        }
    }
}

impl From<&IntegrationNodeQueryNodeOnIntegrationFileTreeEntries> for IntegrationEntry {
    fn from(entry: &IntegrationNodeQueryNodeOnIntegrationFileTreeEntries) -> Self {
        IntegrationEntry {
            fqn: entry.fqn.clone(),
            id: entry.id.clone(),
            name: entry.name.clone(),
        }
    }
}

impl IntegrationNode {
    /// Method to get a potential match in this node, for an entry matching the name.
    pub fn get_id_for_name(&self, name: String) -> Option<String> {
        let mut result = None;
        for e in self.entries.iter() {
            if name == e.name {
                result = Some(e.id.clone());
                break;
            }
        }
        result
    }
}

/// The `Integrations` structure implements the `IntegrationsIntf` to get the information from
/// the GraphQL server.
pub struct Integrations {}

impl Integrations {
    pub fn new() -> Self {
        Self {}
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
        let int_type_name = int_type.unwrap_or_default();

        // walk the list of integration details looking for matches (and duplicates)
        for entry in all_entries {
            if int_name.unwrap() != entry.name.as_str() {
                continue;
            }

            if int_type_name.is_empty() {
                integration_types.push(entry.integration_type.clone());
                found = Some(entry);
            } else if int_type_name.to_lowercase() == entry.integration_type.to_lowercase() {
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

    fn get_integration_node(&self, entry_id: String) -> GraphQLResult<Option<IntegrationNode>> {
        let query =
            IntegrationNodeQuery::build_query(integration_node_query::Variables { entry_id });
        let response_body = graphql_request::<_, integration_node_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(node) = data.node {
                if let AwsIntegration(aws) = &node.on {
                    Ok(Some(IntegrationNode::from(aws)))
                } else if let GithubIntegration(gh) = &node.on {
                    Ok(Some(IntegrationNode::from(gh)))
                } else if let IntegrationServiceTree(st) = &node.on {
                    Ok(Some(IntegrationNode::from(st)))
                } else if let IntegrationFileTree(st) = &node.on {
                    Ok(Some(IntegrationNode::from(st)))
                } else if let IntegrationFile(f) = &node.on {
                    Ok(Some(IntegrationNode::from(f)))
                } else {
                    dbg!(&node);
                    panic!("Unhandled type")
                }
            } else {
                Ok(None)
            }
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

    #[test]
    fn get_keys_empty() {
        let json = "{}";
        let value = serde_json::from_str::<Value>(&json).unwrap();
        let expected: Vec<String> = Vec::new();
        assert_eq!(get_keys(value), expected);
    }

    #[test]
    fn get_keys_simple() {
        let json = r#"{"a":"b", "c":"d", "e":1}"#;
        let value = serde_json::from_str::<Value>(&json).unwrap();
        let expected: Vec<String> = vec!["a".to_string(), "c".to_string(), "e".to_string()];
        assert_eq!(get_keys(value), expected);
    }

    #[test]
    fn get_keys_moderate() {
        let json = r#"{ "a": ["an", "array"], "b": { "an": "object" } }"#;
        let value = serde_json::from_str::<Value>(&json).unwrap();
        let expected: Vec<String> = vec!["a".to_string(), "b.an".to_string()];
        assert_eq!(get_keys(value), expected);
    }
    #[test]
    fn get_keys_complex() {
        let json =
            r#"{"a":"b", "c":"d", "e": {"f":"g", "h": {"i":"j", "k":"l"}, "m":"n"}, "o":"p"}"#;
        let value = serde_json::from_str::<Value>(&json).unwrap();
        let expected: Vec<String> = vec![
            "a".to_string(),
            "c".to_string(),
            "e.f".to_string(),
            "e.h.i".to_string(),
            "e.h.k".to_string(),
            "e.m".to_string(),
            "o".to_string(),
        ];
        assert_eq!(get_keys(value), expected);
    }
}
