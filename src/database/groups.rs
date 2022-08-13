use cloudtruth_restapi::apis::groups_api::{
    groups_add_create, groups_create, groups_destroy, groups_list, groups_partial_update,
    groups_remove_create,
};
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{Group, PatchedGroup};

use crate::database::{auth_details, page_size, response_message, GroupError, OpenApiConfig};

const NO_ORDERING: Option<&str> = None;

pub struct Groups {}

fn auth_error(content: &str) -> GroupError {
    GroupError::Authentication(auth_details(content))
}

fn response_error(status: &reqwest::StatusCode, content: &str) -> GroupError {
    match status.as_u16() {
        401 => auth_error(content),
        403 => auth_error(content),
        _ => GroupError::ResponseError(response_message(status, content)),
    }
}

impl Groups {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_group_details(&self, rest_cfg: &OpenApiConfig) -> Result<Vec<Group>, GroupError> {
        let mut page_count = 1;
        let mut result: Vec<Group> = Vec::new();
        loop {
            let response = groups_list(
                rest_cfg,
                None,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
            );
            match response {
                Ok(paginated_group_list) => {
                    if let Some(groups) = paginated_group_list.results {
                        for group in groups {
                            result.push(group);
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if paginated_group_list.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(GroupError::UnhandledError(e.to_string())),
            }
        }
        Ok(result)
    }

    pub fn get_group_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        group_name: &str,
    ) -> Result<Option<Group>, GroupError> {
        let page_count = 1;
        let response = groups_list(
            rest_cfg,
            Some(group_name),
            NO_ORDERING,
            Some(page_count),
            page_size(rest_cfg),
            None,
        );
        match response {
            Ok(paginated_group_list) => {
                let group = paginated_group_list
                    .results
                    .and_then(|mut groups| groups.pop());
                Ok(group)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        group_name: &str,
    ) -> Result<Option<String>, GroupError> {
        Ok(self
            .get_group_details_by_name(rest_cfg, group_name)?
            .map(|g| g.id))
    }

    pub fn delete_group(&self, rest_cfg: &OpenApiConfig, group_id: &str) -> Result<(), GroupError> {
        let response = groups_destroy(rest_cfg, group_id);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }

    pub fn create_group(
        &self,
        rest_cfg: &OpenApiConfig,
        group_name: &str,
        description: Option<&str>,
    ) -> Result<Group, GroupError> {
        let response = groups_create(
            rest_cfg,
            Group {
                name: group_name.to_string(),
                description: description.map(str::to_string),
                id: String::default(),
                url: String::default(),
                created_at: String::default(),
                modified_at: String::default(),
                users: Vec::new(),
            },
        );
        match response {
            Ok(group) => Ok(group),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }

    pub fn update_group(
        &self,
        rest_cfg: &OpenApiConfig,
        group_id: &str,
        group_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), GroupError> {
        let group_update = PatchedGroup {
            name: group_name.map(str::to_string),
            description: description.map(str::to_string),
            id: None,
            url: None,
            created_at: None,
            modified_at: None,
            users: None,
        };
        let response = groups_partial_update(rest_cfg, group_id, Some(group_update));
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }

    pub fn add_user_to_group(
        &self,
        rest_cfg: &OpenApiConfig,
        group: &Group,
        user_url: &str,
    ) -> Result<(), GroupError> {
        /* Need to decode urlencoding from the user URL to avoid double-encoding in the REST client */
        let user_url_decoded = urlencoding::decode(user_url).unwrap();
        let response = groups_add_create(rest_cfg, &group.id, &user_url_decoded, group.clone());
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }
    pub fn remove_user_from_group(
        &self,
        rest_cfg: &OpenApiConfig,
        group: &Group,
        user_url: &str,
    ) -> Result<(), GroupError> {
        /* Need to decode urlencoding from the user URL to avoid double-encoding in the REST client */
        let user_url_decoded = urlencoding::decode(user_url).unwrap();
        let response = groups_remove_create(rest_cfg, &group.id, &user_url_decoded, group.clone());
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(GroupError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_property(&self, group: &Group, property_name: &str) -> String {
        match property_name {
            "name" => group.name.clone(),
            "description" => group.description.clone().unwrap_or_default(),
            "users" => group.users.join(", "),
            "created-at" => group.created_at.clone(),
            "modified-at" => group.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, group: &Group, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(group, p)).collect()
    }
}
