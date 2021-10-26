use crate::database::{
    auth_details, extract_details, response_message, IntegrationDetails, IntegrationError,
    IntegrationNode, OpenApiConfig, PAGE_SIZE,
};
use cloudtruth_restapi::apis::integrations_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;

const NO_ORDERING: Option<&str> = None;

fn response_error(status: &reqwest::StatusCode, content: &str) -> IntegrationError {
    match status.as_u16() {
        401 => auth_error(content),
        403 => auth_error(content),
        _ => IntegrationError::ResponseError(response_message(status, content)),
    }
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

    fn get_aws_integration_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<IntegrationDetails>, IntegrationError> {
        let response = integrations_aws_list(rest_cfg, None, None, NO_ORDERING, None, PAGE_SIZE);
        match response {
            Ok(data) => {
                let mut result: Vec<IntegrationDetails> = Vec::new();
                if let Some(list) = data.results {
                    for gh in list {
                        result.push(IntegrationDetails::from(&gh));
                    }
                }
                Ok(result)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    fn get_github_integration_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<IntegrationDetails>, IntegrationError> {
        let response = integrations_github_list(rest_cfg, None, NO_ORDERING, None, PAGE_SIZE);
        match response {
            Ok(data) => {
                let mut result: Vec<IntegrationDetails> = Vec::new();
                if let Some(list) = data.results {
                    for gh in list {
                        result.push(IntegrationDetails::from(&gh));
                    }
                }
                Ok(result)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    /// Gets a list of `IntegrationDetails` for all integration types.
    pub fn get_integration_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<IntegrationDetails>, IntegrationError> {
        let mut github_details = self.get_github_integration_details(rest_cfg)?;
        let mut aws_details = self.get_aws_integration_details(rest_cfg)?;
        let mut total = vec![];
        total.append(&mut github_details);
        total.append(&mut aws_details);
        Ok(total)
    }

    /// Get the integration node by FQN
    pub fn get_integration_nodes(
        &self,
        rest_cfg: &OpenApiConfig,
        fqn: Option<&str>,
    ) -> Result<Vec<IntegrationNode>, IntegrationError> {
        let response = integrations_explore_list(rest_cfg, fqn, NO_ORDERING, None, PAGE_SIZE);
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
