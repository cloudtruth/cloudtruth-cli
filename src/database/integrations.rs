use crate::database::{
    auth_details, extract_details, last_from_url, parent_id_from_url, response_message,
    ActionDetails, IntegrationDetails, IntegrationError, IntegrationNode, OpenApiConfig,
    TaskDetail, PAGE_SIZE,
};
use cloudtruth_restapi::apis::integrations_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{AwsPull, AwsPush, AwsPushUpdate, AwsRegionEnum, AwsServiceEnum};

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

fn aws_region_from_str(input: &str) -> Option<AwsRegionEnum> {
    match input.to_lowercase().as_str() {
        "af-south-1" => Some(AwsRegionEnum::AfSouth1),
        "ap-east-1" => Some(AwsRegionEnum::ApEast1),
        "ap-northeast-1" => Some(AwsRegionEnum::ApNortheast1),
        "ap-northeast-2" => Some(AwsRegionEnum::ApNortheast2),
        "ap-northeast-3" => Some(AwsRegionEnum::ApNortheast3),
        "ap-south-1" => Some(AwsRegionEnum::ApSouth1),
        "ap-southeast-1" => Some(AwsRegionEnum::ApSoutheast1),
        "ap-southeast-2" => Some(AwsRegionEnum::ApSoutheast2),
        "ca-central-1" => Some(AwsRegionEnum::CaCentral1),
        "cn-north-1" => Some(AwsRegionEnum::CnNorth1),
        "cn-northwest-1" => Some(AwsRegionEnum::CnNorthwest1),
        "eu-central-1" => Some(AwsRegionEnum::EuCentral1),
        "eu-north-1" => Some(AwsRegionEnum::EuNorth1),
        "eu-south-1" => Some(AwsRegionEnum::EuSouth1),
        "eu-west-1" => Some(AwsRegionEnum::EuWest1),
        "eu-west-2" => Some(AwsRegionEnum::EuWest2),
        "eu-west-3" => Some(AwsRegionEnum::EuWest3),
        "me-south-1" => Some(AwsRegionEnum::MeSouth1),
        "sa-east-1" => Some(AwsRegionEnum::SaEast1),
        "us-east-1" => Some(AwsRegionEnum::UsEast1),
        "us-east-2" => Some(AwsRegionEnum::UsEast2),
        "us-west-1" => Some(AwsRegionEnum::UsWest1),
        "us-west-2" => Some(AwsRegionEnum::UsWest2),
        _ => None,
    }
}

fn aws_service_from_str(input: &str) -> Option<AwsServiceEnum> {
    match input.to_lowercase().as_str() {
        "s3" => Some(AwsServiceEnum::S3),
        "secretsmanager" => Some(AwsServiceEnum::Secretsmanager),
        "ssm" => Some(AwsServiceEnum::Ssm),
        _ => None,
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
            let name = last_from_url(fqn);
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

    fn get_aws_id(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<String>, IntegrationError> {
        // unfortunately, there's no good way to filter by name on the server... so get the whole
        // list and filter here
        let mut total = self.get_aws_integration_details(rest_cfg)?;
        total.retain(|d| d.name == integration_name);
        match total.len() {
            0 => Ok(None),
            _ => Ok(Some(total[0].id.clone())),
        }
    }

    fn get_gitub_id(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<String>, IntegrationError> {
        // unfortunately, there's no good way to filter by name on the server... so get the whole
        // list and filter here
        let mut total = self.get_github_integration_details(rest_cfg)?;
        total.retain(|d| d.name == integration_name);
        match total.len() {
            0 => Ok(None),
            _ => Ok(Some(total[0].id.clone())),
        }
    }

    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<String>, IntegrationError> {
        match self.get_gitub_id(rest_cfg, integration_name)? {
            Some(github_id) => Ok(Some(github_id)),
            _ => match self.get_aws_id(rest_cfg, integration_name)? {
                Some(aws_id) => Ok(Some(aws_id)),
                _ => Ok(None),
            },
        }
    }

    fn get_aws_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<IntegrationDetails>, IntegrationError> {
        // unfortunately, there's no good way to filter by name on the server... so get the whole
        // list and filter here
        let mut total = self.get_aws_integration_details(rest_cfg)?;
        total.retain(|d| d.name == integration_name);
        match total.len() {
            0 => Ok(None),
            _ => Ok(Some(total[0].clone())),
        }
    }

    fn get_gitub_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<IntegrationDetails>, IntegrationError> {
        // unfortunately, there's no good way to filter by name on the server... so get the whole
        // list and filter here
        let mut total = self.get_github_integration_details(rest_cfg)?;
        total.retain(|d| d.name == integration_name);
        match total.len() {
            0 => Ok(None),
            _ => Ok(Some(total[0].clone())),
        }
    }

    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_name: &str,
    ) -> Result<Option<IntegrationDetails>, IntegrationError> {
        match self.get_gitub_details_by_name(rest_cfg, integration_name)? {
            Some(github_details) => Ok(Some(github_details)),
            _ => match self.get_aws_details_by_name(rest_cfg, integration_name)? {
                Some(aws_details) => Ok(Some(aws_details)),
                _ => Ok(None),
            },
        }
    }

    fn refresh_aws_connection(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        let response = integrations_aws_retrieve(rest_cfg, integration_id, Some(true));
        match response {
            Ok(api) => Ok(Some(api.id)),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                404 => Ok(None),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    fn refresh_github_connection(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        let response = integrations_github_retrieve(rest_cfg, integration_id, Some(true));
        match response {
            Ok(api) => Ok(Some(api.id)),
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                404 => Ok(None),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn refresh_connection(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
    ) -> Result<(), IntegrationError> {
        self.refresh_github_connection(rest_cfg, integration_id)?;
        self.refresh_aws_connection(rest_cfg, integration_id)?;
        Ok(())
    }

    ///==========================================
    /// Integration push
    ///==========================================
    fn get_aws_push_list(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        name: Option<&str>,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        let response = integrations_aws_pushes_list(
            rest_cfg,
            integration_id,
            None,
            name,
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );
        match response {
            Ok(data) => {
                let mut result: Vec<ActionDetails> = Vec::new();
                if let Some(list) = data.results {
                    for api in list {
                        result.push(ActionDetails::from(&api));
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

    pub fn get_push_list(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_aws_push_list(rest_cfg, integration_id, None)
    }

    fn get_all_aws_pushes(
        &self,
        rest_cfg: &OpenApiConfig,
        name: Option<&str>,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        let int_details = self.get_aws_integration_details(rest_cfg)?;
        let mut total: Vec<ActionDetails> = vec![];
        for entry in int_details {
            let mut pushes = self.get_aws_push_list(rest_cfg, &entry.id, name)?;
            for p in &mut pushes {
                p.integration_name = entry.name.clone();
            }
            total.append(&mut pushes);
        }
        Ok(total)
    }

    pub fn get_all_pushes(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_all_aws_pushes(rest_cfg, None)
    }

    pub fn get_all_pushes_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        push_name: &str,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_all_aws_pushes(rest_cfg, Some(push_name))
    }

    fn get_aws_push_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_name: &str,
    ) -> Result<Option<ActionDetails>, IntegrationError> {
        let response = integrations_aws_pushes_list(
            rest_cfg,
            integration_id,
            None,
            Some(push_name),
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );
        match response {
            Ok(data) => match data.results {
                Some(list) => match list.is_empty() {
                    true => Ok(None),
                    _ => Ok(Some(ActionDetails::from(&list[0]))),
                },
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_push_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_name: &str,
    ) -> Result<Option<ActionDetails>, IntegrationError> {
        self.get_aws_push_by_name(rest_cfg, integration_id, push_name)
    }

    fn get_aws_push_tasks(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
    ) -> Result<Vec<TaskDetail>, IntegrationError> {
        let response = integrations_aws_pushes_tasks_list(
            rest_cfg,
            integration_id,
            push_id,
            None,
            None,
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
            None,
        );
        match response {
            Ok(data) => {
                let mut result: Vec<TaskDetail> = Vec::new();
                if let Some(list) = data.results {
                    for api in list {
                        result.push(TaskDetail::from(&api));
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

    pub fn get_push_tasks(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
    ) -> Result<Vec<TaskDetail>, IntegrationError> {
        let mut total: Vec<TaskDetail> = vec![];
        let mut aws_tasks = self.get_aws_push_tasks(rest_cfg, integration_id, push_id)?;
        total.append(&mut aws_tasks);
        Ok(total)
    }

    fn delete_aws_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        let response = integrations_aws_pushes_destroy(rest_cfg, integration_id, push_id);
        match response {
            Ok(_) => Ok(Some(push_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        self.delete_aws_push(rest_cfg, integration_id, push_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_aws_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_name: &str,
        resource: &str,
        region: &str,
        service: &str,
        description: Option<&str>,
        projects: Vec<String>,
        tags: Vec<String>,
    ) -> Result<ActionDetails, IntegrationError> {
        let reg_enum = aws_region_from_str(region).unwrap_or(AwsRegionEnum::UsEast1);
        let ser_enum = aws_service_from_str(service).unwrap_or(AwsServiceEnum::Ssm);
        let push_create = AwsPush {
            url: "".to_string(),
            id: "".to_string(),
            name: push_name.to_string(),
            description: description.map(String::from),
            projects,
            tags,
            region: Box::new(reg_enum),
            service: Box::new(ser_enum),
            resource: resource.to_string(),
            latest_task: None,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let response = integrations_aws_pushes_create(rest_cfg, integration_id, push_create);
        match response {
            Ok(api) => Ok(ActionDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_name: &str,
        resource: &str,
        region: &str,
        service: &str,
        description: Option<&str>,
        projects: Vec<String>,
        tags: Vec<String>,
    ) -> Result<ActionDetails, IntegrationError> {
        self.create_aws_push(
            rest_cfg,
            integration_id,
            push_name,
            resource,
            region,
            service,
            description,
            projects,
            tags,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_aws_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
        push_name: &str,
        resource: &str,
        description: Option<&str>,
        projects: Vec<String>,
        tags: Vec<String>,
    ) -> Result<(), IntegrationError> {
        let push_update = AwsPushUpdate {
            name: push_name.to_string(),
            description: description.map(String::from),
            projects,
            tags,
            resource: resource.to_string(),
        };
        let response =
            integrations_aws_pushes_update(rest_cfg, integration_id, push_id, push_update);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_push(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        push_id: &str,
        push_name: &str,
        resource: &str, // NOTE: unfortunately, this needs to be specified each time
        description: Option<&str>,
        projects: Vec<String>,
        tags: Vec<String>,
    ) -> Result<(), IntegrationError> {
        self.update_aws_push(
            rest_cfg,
            integration_id,
            push_id,
            push_name,
            resource,
            description,
            projects,
            tags,
        )
    }

    fn sync_aws_push(
        &self,
        rest_cfg: &OpenApiConfig,
        push_details: &ActionDetails,
    ) -> Result<(), IntegrationError> {
        let description = if push_details.description.is_empty() {
            None
        } else {
            Some(push_details.description.clone())
        };
        let reg_enum = aws_region_from_str(&push_details.region).unwrap();
        let srv_enum = aws_service_from_str(&push_details.service).unwrap();
        let integration_id = parent_id_from_url(&push_details.url, "pushes/");
        let sync_body = AwsPush {
            url: push_details.url.clone(),
            id: push_details.id.clone(),
            name: push_details.name.clone(),
            description,
            projects: push_details.project_urls.clone(),
            tags: push_details.tag_urls.clone(),
            region: Box::new(reg_enum),
            service: Box::new(srv_enum),
            resource: push_details.resource.clone(),
            latest_task: None,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let response = integrations_aws_pushes_sync_create(
            rest_cfg,
            integration_id,
            &push_details.id,
            sync_body,
        );
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn sync_push(
        &self,
        rest_cfg: &OpenApiConfig,
        push_details: &ActionDetails,
    ) -> Result<(), IntegrationError> {
        self.sync_aws_push(rest_cfg, push_details)
    }

    ///==========================================
    /// Integration pulls
    ///==========================================
    fn get_aws_pull_list(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        name: Option<&str>,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        let response = integrations_aws_pulls_list(
            rest_cfg,
            integration_id,
            None,
            name,
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );
        match response {
            Ok(data) => {
                let mut result: Vec<ActionDetails> = Vec::new();
                if let Some(list) = data.results {
                    for api in list {
                        result.push(ActionDetails::from(&api));
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

    pub fn get_pull_list(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_aws_pull_list(rest_cfg, integration_id, None)
    }

    fn get_all_aws_pulls(
        &self,
        rest_cfg: &OpenApiConfig,
        name: Option<&str>,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        let int_details = self.get_aws_integration_details(rest_cfg)?;
        let mut total: Vec<ActionDetails> = vec![];
        for entry in int_details {
            let mut pulls = self.get_aws_pull_list(rest_cfg, &entry.id, name)?;
            for p in &mut pulls {
                p.integration_name = entry.name.clone();
            }
            total.append(&mut pulls);
        }
        Ok(total)
    }

    pub fn get_all_pulls(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_all_aws_pulls(rest_cfg, None)
    }

    pub fn get_all_pulls_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        name: &str,
    ) -> Result<Vec<ActionDetails>, IntegrationError> {
        self.get_all_aws_pulls(rest_cfg, Some(name))
    }

    fn get_aws_pull_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_name: &str,
    ) -> Result<Option<ActionDetails>, IntegrationError> {
        let response = integrations_aws_pulls_list(
            rest_cfg,
            integration_id,
            None,
            Some(pull_name),
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
        );
        match response {
            Ok(data) => match data.results {
                Some(list) => match list.is_empty() {
                    true => Ok(None),
                    _ => Ok(Some(ActionDetails::from(&list[0]))),
                },
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_pull_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_name: &str,
    ) -> Result<Option<ActionDetails>, IntegrationError> {
        self.get_aws_pull_by_name(rest_cfg, integration_id, pull_name)
    }

    fn get_aws_pull_tasks(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
    ) -> Result<Vec<TaskDetail>, IntegrationError> {
        let response = integrations_aws_pulls_tasks_list(
            rest_cfg,
            integration_id,
            pull_id,
            None,
            None,
            None,
            NO_ORDERING,
            None,
            PAGE_SIZE,
            None,
        );
        match response {
            Ok(data) => {
                let mut result: Vec<TaskDetail> = Vec::new();
                if let Some(list) = data.results {
                    for api in list {
                        result.push(TaskDetail::from(&api));
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

    pub fn get_pull_tasks(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
    ) -> Result<Vec<TaskDetail>, IntegrationError> {
        let mut total: Vec<TaskDetail> = vec![];
        let mut aws_tasks = self.get_aws_pull_tasks(rest_cfg, integration_id, pull_id)?;
        total.append(&mut aws_tasks);
        Ok(total)
    }

    fn delete_aws_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        let response = integrations_aws_pulls_destroy(rest_cfg, integration_id, pull_id);
        match response {
            Ok(_) => Ok(Some(pull_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
    ) -> Result<Option<String>, IntegrationError> {
        self.delete_aws_pull(rest_cfg, integration_id, pull_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_aws_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_name: &str,
        resource: &str,
        region: &str,
        service: &str,
        description: Option<&str>,
        dry_run: Option<bool>,
    ) -> Result<ActionDetails, IntegrationError> {
        let reg_enum = aws_region_from_str(region).unwrap_or(AwsRegionEnum::UsEast1);
        let ser_enum = aws_service_from_str(service).unwrap_or(AwsServiceEnum::Ssm);
        let pull_create = AwsPull {
            url: "".to_string(),
            id: "".to_string(),
            name: pull_name.to_string(),
            description: description.map(String::from),
            region: Box::new(reg_enum),
            service: Box::new(ser_enum),
            resource: resource.to_string(),
            latest_task: None,
            created_at: "".to_string(),
            modified_at: "".to_string(),
            dry_run,
        };
        let response = integrations_aws_pulls_create(rest_cfg, integration_id, pull_create);
        match response {
            Ok(api) => Ok(ActionDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_name: &str,
        resource: &str,
        region: &str,
        service: &str,
        description: Option<&str>,
        dry_run: Option<bool>,
    ) -> Result<ActionDetails, IntegrationError> {
        self.create_aws_pull(
            rest_cfg,
            integration_id,
            pull_name,
            resource,
            region,
            service,
            description,
            dry_run,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_aws_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
        pull_name: &str,
        resource: &str,
        description: Option<&str>,
        dry_run: Option<bool>,
    ) -> Result<(), IntegrationError> {
        let pull_update = AwsPull {
            url: "".to_string(),
            id: "".to_string(),
            name: pull_name.to_string(),
            description: description.map(String::from),
            latest_task: None,
            created_at: "".to_string(),
            modified_at: "".to_string(),
            dry_run,
            // TODO: can these be updated? will things barf if they're changed
            region: Box::new(AwsRegionEnum::AfSouth1),
            service: Box::new(AwsServiceEnum::S3),
            resource: resource.to_string(),
        };
        let response =
            integrations_aws_pulls_update(rest_cfg, integration_id, pull_id, pull_update);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        integration_id: &str,
        pull_id: &str,
        pull_name: &str,
        resource: &str, // NOTE: unfortunately, this needs to be specified each time
        description: Option<&str>,
        dry_run: Option<bool>,
    ) -> Result<(), IntegrationError> {
        self.update_aws_pull(
            rest_cfg,
            integration_id,
            pull_id,
            pull_name,
            resource,
            description,
            dry_run,
        )
    }

    fn sync_aws_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        pull_details: &ActionDetails,
    ) -> Result<(), IntegrationError> {
        let description = if pull_details.description.is_empty() {
            None
        } else {
            Some(pull_details.description.clone())
        };
        let reg_enum = aws_region_from_str(&pull_details.region).unwrap();
        let srv_enum = aws_service_from_str(&pull_details.service).unwrap();
        let integration_id = parent_id_from_url(&pull_details.url, "pulls/");
        let sync_body = AwsPull {
            url: pull_details.url.clone(),
            id: pull_details.id.clone(),
            name: pull_details.name.clone(),
            description,
            region: Box::new(reg_enum),
            service: Box::new(srv_enum),
            resource: pull_details.resource.clone(),
            latest_task: None,
            created_at: "".to_string(),
            modified_at: "".to_string(),
            dry_run: None,
        };
        let response = integrations_aws_pulls_sync_create(
            rest_cfg,
            integration_id,
            &pull_details.id,
            sync_body,
        );
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(IntegrationError::UnhandledError(e.to_string())),
        }
    }

    pub fn sync_pull(
        &self,
        rest_cfg: &OpenApiConfig,
        pull_details: &ActionDetails,
    ) -> Result<(), IntegrationError> {
        self.sync_aws_pull(rest_cfg, pull_details)
    }
}
