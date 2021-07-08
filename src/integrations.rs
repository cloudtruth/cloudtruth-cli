use crate::openapi::open_api_config;
use cloudtruth_restapi::apis::integrations_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{AwsIntegration, GitHubIntegration, IntegrationExplorer};

#[derive(Debug)]
pub struct IntegrationDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub fqn: String,
    pub status: String,
    pub status_detail: String,
    pub status_time: String,
}

impl From<&AwsIntegration> for IntegrationDetails {
    fn from(aws: &AwsIntegration) -> Self {
        IntegrationDetails {
            id: aws.id.clone(),
            provider: "aws".to_string(),
            name: aws.name.clone(),
            description: aws.description.clone().unwrap_or_default(),
            fqn: aws.fqn.clone(),
            status: aws.status.clone(),
            status_detail: aws.status_detail.clone(),
            status_time: aws.status_last_checked_at.clone(),
        }
    }
}

impl From<&GitHubIntegration> for IntegrationDetails {
    fn from(github: &GitHubIntegration) -> Self {
        IntegrationDetails {
            id: github.id.clone(),
            provider: "github".to_string(),
            name: github.name.clone(),
            description: github.description.clone().unwrap_or_default(),
            fqn: github.fqn.clone(),
            status: github.status.clone(),
            status_detail: github.status_detail.clone(),
            status_time: github.status_last_checked_at.clone(),
        }
    }
}

#[derive(Debug)]
pub struct IntegrationNode {
    pub fqn: String,
    pub node_type: String,
    pub secret: bool,
    pub name: String,
    pub content_type: String,
    pub content_size: i32,
    pub content_data: String,
    pub content_keys: Vec<String>,
}

fn get_name(name: &Option<String>, fqn: &str) -> String {
    if let Some(name) = name {
        name.clone()
    } else {
        fqn.split('/').last().unwrap().to_string()
    }
}

impl From<&IntegrationExplorer> for IntegrationNode {
    fn from(node: &IntegrationExplorer) -> Self {
        IntegrationNode {
            fqn: node.fqn.clone(),
            name: get_name(&node.name, &node.fqn),
            node_type: format!("{:?}", node.node_type),
            secret: node.secret.unwrap_or(false),
            content_type: node.content_type.clone().unwrap_or_default(),
            content_size: node.content_size.clone().unwrap_or(0),
            content_data: node.content_data.clone().unwrap_or_default(),
            content_keys: node.content_keys.clone().unwrap_or_default(),
        }
    }
}

/// This is the interface that is implemented to retrieve integration information.
///
/// This layer of abstraction is done to allow for mocking in unittest, and to potentially allow
/// for future implementations.
pub trait IntegrationsIntf {
    /// Gets a list of `IntegrationDetails` for all integration types.
    fn get_integration_details(
        &self,
    ) -> Result<Vec<IntegrationDetails>, Error<IntegrationsGithubListError>>;

    /// Get the integration node by FQN
    fn get_integration_nodes(
        &self,
        fqn: Option<&str>,
    ) -> Result<Vec<IntegrationNode>, Error<IntegrationsExploreListError>>;
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
    ) -> Result<Vec<IntegrationDetails>, Error<IntegrationsGithubListError>> {
        let mut result: Vec<IntegrationDetails> = Vec::new();
        let rest_cfg = open_api_config();

        let response = integrations_github_list(&rest_cfg, None, None);
        if let Ok(paged_results) = response {
            if let Some(list) = paged_results.results {
                for gh in list {
                    result.push(IntegrationDetails::from(&gh));
                }
            }
        }

        let response = integrations_aws_list(&rest_cfg, None, None, None);
        if let Ok(paged_results) = response {
            if let Some(list) = paged_results.results {
                for aws in list {
                    result.push(IntegrationDetails::from(&aws));
                }
            }
        }

        Ok(result)
    }

    fn get_integration_nodes(
        &self,
        fqn: Option<&str>,
    ) -> Result<Vec<IntegrationNode>, Error<IntegrationsExploreListError>> {
        let rest_cfg = open_api_config();
        let mut results: Vec<IntegrationNode> = Vec::new();
        let response = integrations_explore_list(&rest_cfg, fqn, None);
        if let Ok(response) = response {
            if let Some(list) = response.results {
                for item in list {
                    results.push(IntegrationNode::from(&item))
                }
                results.sort_by(|l, r| l.fqn.cmp(&r.fqn));
            }
        } else if let Some(fqn) = fqn {
            if let Err(ResponseError(content)) = response {
                let name = fqn
                    .split('/')
                    .filter(|&x| !x.is_empty())
                    .last()
                    .unwrap_or_default();
                let err_msg = if let Some(entity) = content.entity {
                    format!("{:?}", entity)
                } else {
                    "".to_owned()
                };
                if content.status == 415 {
                    results.push(IntegrationNode {
                        fqn: fqn.to_string(),
                        node_type: "FILE".to_owned(),
                        secret: false,
                        name: name.to_string(),
                        content_type: "application/binary".to_owned(),
                        content_size: 0,
                        content_data: err_msg,
                        content_keys: vec![],
                    })
                } else if content.status == 507 {
                    results.push(IntegrationNode {
                        fqn: fqn.to_string(),
                        node_type: "FILE".to_owned(),
                        secret: false,
                        name: name.to_string(),
                        content_type: "".to_owned(),
                        content_size: 0,
                        content_data: err_msg,
                        content_keys: vec![],
                    });
                }
            }
        }
        Ok(results)
    }
}
