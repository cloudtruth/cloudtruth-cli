use crate::database::{
    extract_details, page_size, response_message, OpenApiConfig, ParamRuleType, TypeDetails,
    TypeError, NO_PAGE_COUNT, NO_PAGE_SIZE,
};

use cloudtruth_restapi::apis::types_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    ParameterRuleTypeEnum, ParameterTypeCreate, ParameterTypeRuleCreate,
    PatchedParameterTypeRuleUpdate, PatchedParameterTypeUpdate,
};
use std::result::Result;

const NO_DESC_CONTAINS: Option<&str> = None;
const NO_NAME_CONTAINS: Option<&str> = None;
const NO_ORDERING: Option<&str> = None;

pub struct Types {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> TypeError {
    TypeError::ResponseError(response_message(status, content))
}

fn rule_error(action: String, content: &str) -> TypeError {
    TypeError::RuleViolation(action, extract_details(content))
}

impl Types {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the details for `type_name`
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        type_name: &str,
    ) -> Result<Option<TypeDetails>, TypeError> {
        let response = types_list(
            rest_cfg,
            NO_DESC_CONTAINS,
            Some(type_name),
            NO_NAME_CONTAINS,
            NO_ORDERING,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
        );

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let api_type = &list[0];
                        let details = TypeDetails::from(api_type);
                        Ok(Some(details))
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    /// Resolve the `type_name` to a String
    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        type_name: &str,
    ) -> Result<Option<String>, TypeError> {
        if let Some(details) = self.get_details_by_name(rest_cfg, type_name)? {
            Ok(Some(details.id))
        } else {
            Ok(None)
        }
    }

    /// Get a complete list of types for this organization.
    pub fn get_type_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<TypeDetails>, TypeError> {
        let mut types: Vec<TypeDetails> = vec![];
        let mut page_count = 1;
        loop {
            let response = types_list(
                rest_cfg,
                NO_DESC_CONTAINS,
                None,
                NO_NAME_CONTAINS,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
            );

            match response {
                Ok(data) => {
                    if let Some(list) = data.results {
                        for api_prj in list {
                            let details = TypeDetails::from(&api_prj);
                            types.push(details);
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() || data.next.as_ref().unwrap().is_empty() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(TypeError::UnhandledError(e.to_string())),
            }
        }
        Ok(types)
    }

    /// Create a type with the specified name/description
    pub fn create_type(
        &self,
        rest_cfg: &OpenApiConfig,
        type_name: &str,
        description: Option<&str>,
        parent_url: &str,
    ) -> Result<TypeDetails, TypeError> {
        let create_type = ParameterTypeCreate {
            name: type_name.to_string(),
            description: description.map(String::from),
            parent: parent_url.to_string(),
        };
        let response = types_create(rest_cfg, create_type);
        match response {
            // return the type id of the newly minted type
            Ok(param_type) => Ok(TypeDetails::from(&param_type)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    /// Delete the specified type
    pub fn delete_type(
        &self,
        rest_cfg: &OpenApiConfig,
        type_id: &str,
    ) -> Result<Option<String>, TypeError> {
        let response = types_destroy(rest_cfg, type_id);
        match response {
            Ok(_) => Ok(Some(type_id.to_string())),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    /// Update the specified type
    pub fn update_type(
        &self,
        rest_cfg: &OpenApiConfig,
        type_name: &str,
        type_id: &str,
        description: Option<&str>,
        parent_url: Option<&str>,
    ) -> Result<TypeDetails, TypeError> {
        let type_update = PatchedParameterTypeUpdate {
            id: None,
            name: Some(type_name.to_string()),
            description: description.map(|d| d.to_string()),
            created_at: None,
            modified_at: None,
            parent: parent_url.map(String::from),
        };
        let response = types_partial_update(rest_cfg, type_id, Some(type_update));
        match response {
            Ok(param_type) => {
                let details = TypeDetails::from(&param_type);
                Ok(details)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    pub fn create_type_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        type_id: &str,
        rule_type: ParamRuleType,
        constraint: &str,
    ) -> Result<String, TypeError> {
        let rule_create = ParameterTypeRuleCreate {
            _type: ParameterRuleTypeEnum::from(rule_type),
            constraint: constraint.to_string(),
        };
        let response = types_rules_create(rest_cfg, type_id, rule_create);
        let action = "create".to_string();
        match response {
            Ok(rule) => Ok(rule.id),
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    pub fn update_type_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        type_id: &str,
        rule_id: &str,
        rule_type: Option<ParamRuleType>,
        constraint: Option<&str>,
    ) -> Result<String, TypeError> {
        let patch_rule = PatchedParameterTypeRuleUpdate {
            id: None,
            parameter_type: None,
            _type: rule_type.map(ParameterRuleTypeEnum::from),
            constraint: constraint.map(String::from),
            created_at: None,
            modified_at: None,
        };
        let response = types_rules_partial_update(rest_cfg, rule_id, type_id, Some(patch_rule));
        let action = "update".to_string();
        match response {
            Ok(rule) => Ok(rule.id),
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_type_rule(
        &self,
        rest_cfg: &OpenApiConfig,
        type_id: &str,
        rule_id: &str,
    ) -> Result<String, TypeError> {
        let response = types_rules_destroy(rest_cfg, rule_id, type_id);
        let action = "delete".to_string();
        match response {
            Ok(_) => Ok(rule_id.to_string()),
            Err(ResponseError(ref content)) => Err(rule_error(action, &content.content)),
            Err(e) => Err(TypeError::UnhandledError(e.to_string())),
        }
    }
}
