use crate::database::{
    auth_details, extract_details, generic_response_message, IntegrationDetails, IntegrationNode,
    OpenApiConfig, PAGE_SIZE,
};
use cloudtruth_restapi::apis::integrations_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use std::error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum IntegrationError {
    NotFound(String),
    Authentication(String),
    ResponseError(String),
    UnhandledError(String),
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IntegrationError::NotFound(msg) => write!(f, "{}", msg),
            IntegrationError::Authentication(msg) => write!(f, "Not Authenticated: {}", msg),
            IntegrationError::ResponseError(msg) => write!(f, "{}", msg),
            IntegrationError::UnhandledError(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for IntegrationError {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> IntegrationError {
    IntegrationError::ResponseError(generic_response_message(status, content))
}

fn auth_error(content: &str) -> IntegrationError {
    IntegrationError::Authentication(auth_details(content))
}

/// Creates an `IntetgrationNode` for a binary file.
///
/// Marks type as `application/binary` even though this should be returned for
/// any binary type (e.g. jpg, image, mp3). Since an error was thrown, the
/// size is unknown.
fn binary_node(fqn: &str, name: &str, err_msg: &str) -> IntegrationNode {
    IntegrationNode {
        fqn: fqn.to_string(),
        node_type: "FILE".to_owned(),
        secret: false,
        name: name.to_string(),
        content_type: "application/binary".to_owned(),
        content_size: 0,
        content_data: err_msg.to_string(),
        content_keys: vec![],
    }
}

/// Creates an `IntegrationNode` for a large file.
///
/// The `content_type` and `content_size` are undetermined, since the exception
/// does not contain that information.
fn large_node(fqn: &str, name: &str, err_msg: &str) -> IntegrationNode {
    IntegrationNode {
        fqn: fqn.to_string(),
        node_type: "FILE".to_owned(),
        secret: false,
        name: name.to_string(),
        content_type: "".to_owned(),
        content_size: -1,
        content_data: err_msg.to_string(),
        content_keys: vec![],
    }
}

/// The `Integrations` structure implements the `IntegrationsIntf` to get the information from
/// the GraphQL server.
pub struct Integrations {}

impl Integrations {
    pub fn new() -> Self {
        Self {}
    }

    /// Gets a list of `IntegrationDetails` for all integration types.
    pub fn get_integration_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<IntegrationDetails>, IntegrationError> {
        let mut result: Vec<IntegrationDetails> = Vec::new();

        let response = integrations_github_list(rest_cfg, None, None, PAGE_SIZE);
        if let Ok(paged_results) = response {
            if let Some(list) = paged_results.results {
                for gh in list {
                    result.push(IntegrationDetails::from(&gh));
                }
            }
        } else if let Err(ResponseError(ref content)) = response {
            return match content.status.as_u16() {
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            };
        } else {
            return Err(IntegrationError::UnhandledError(
                response.unwrap_err().to_string(),
            ));
        }

        let response = integrations_aws_list(rest_cfg, None, None, None, PAGE_SIZE);
        if let Ok(paged_results) = response {
            if let Some(list) = paged_results.results {
                for aws in list {
                    result.push(IntegrationDetails::from(&aws));
                }
            }
        } else if let Err(ResponseError(ref content)) = response {
            return match content.status.as_u16() {
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            };
        } else {
            return Err(IntegrationError::UnhandledError(
                response.unwrap_err().to_string(),
            ));
        }

        Ok(result)
    }

    /// Get the integration node by FQN
    pub fn get_integration_nodes(
        &self,
        rest_cfg: &OpenApiConfig,
        fqn: Option<&str>,
    ) -> Result<Vec<IntegrationNode>, IntegrationError> {
        let response = integrations_explore_list(rest_cfg, fqn, None, PAGE_SIZE);
        if let Ok(response) = response {
            let mut results: Vec<IntegrationNode> = Vec::new();
            if let Some(list) = response.results {
                for item in list {
                    results.push(IntegrationNode::from(&item))
                }
                results.sort_by(|l, r| l.fqn.cmp(&r.fqn));
            }
            Ok(results)
        } else if let Err(ResponseError(ref content)) = response {
            let fqn = fqn.unwrap_or_default();
            let name = fqn
                .split('/')
                .filter(|&x| !x.is_empty())
                .last()
                .unwrap_or_default();
            let details = extract_details(&content.content);
            if content.status == 415 {
                Ok(vec![binary_node(fqn, name, &details)])
            } else if content.status == 507 {
                Ok(vec![large_node(fqn, name, &details)])
            } else if content.status == 401 || content.status == 403 {
                Err(auth_error(&content.content))
            } else if content.status == 400 || content.status == 404 {
                Err(IntegrationError::NotFound(details))
            } else {
                Err(response_error(&content.status, &content.content))
            }
        } else {
            Err(IntegrationError::UnhandledError(
                response.unwrap_err().to_string(),
            ))
        }
    }
}
