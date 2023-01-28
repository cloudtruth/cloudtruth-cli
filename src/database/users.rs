use crate::database::{
    auth_details, page_size, response_message, OpenApiConfig, UserDetails, UserError,
};
use cloudtruth_restapi::apis::memberships_api::{memberships_create, memberships_partial_update};
use cloudtruth_restapi::apis::serviceaccounts_api::{
    serviceaccounts_create, serviceaccounts_destroy, serviceaccounts_list,
    serviceaccounts_partial_update,
};
use cloudtruth_restapi::apis::users_api::{users_current_retrieve, users_list};
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    MembershipCreate, PatchedMembership, PatchedServiceAccount, RoleEnum,
    ServiceAccountCreateRequest,
};
use std::collections::HashMap;

const NO_ORDERING: Option<&str> = None;

pub type UserNameMap = HashMap<String, String>;

pub struct Users {}

fn auth_error(content: &str) -> UserError {
    UserError::Authentication(auth_details(content))
}

fn response_error(status: &reqwest::StatusCode, content: &str) -> UserError {
    match status.as_u16() {
        401 => auth_error(content),
        403 => auth_error(content),
        _ => UserError::ResponseError(response_message(status, content)),
    }
}

fn to_role_enum(value: &str) -> Result<RoleEnum, UserError> {
    match value.to_uppercase().as_str() {
        "OWNER" => Ok(RoleEnum::OWNER),
        "ADMIN" => Ok(RoleEnum::ADMIN),
        "CONTRIB" => Ok(RoleEnum::CONTRIB),
        "VIEWER" => Ok(RoleEnum::VIEWER),
        _ => Err(UserError::InvalidRole(value.to_string())),
    }
}

impl Users {
    pub fn new() -> Self {
        Self {}
    }

    fn get_service_account_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<UserDetails>, UserError> {
        let mut result: Vec<UserDetails> = Vec::new();
        let mut page_count = 1;
        loop {
            let response =
                serviceaccounts_list(rest_cfg, NO_ORDERING, Some(page_count), page_size(rest_cfg));
            match response {
                Ok(data) => {
                    if let Some(accounts) = data.results {
                        for acct in accounts {
                            result.push(UserDetails::from(&acct));
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
                Err(e) => return Err(UserError::UnhandledError(e.to_string())),
            }
        }
        Ok(result)
    }

    fn get_user_account_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<UserDetails>, UserError> {
        let mut result: Vec<UserDetails> = Vec::new();
        let mut page_count = 1;
        loop {
            let response = users_list(
                rest_cfg,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(accounts) = data.results {
                        for acct in accounts {
                            result.push(UserDetails::from(&acct));
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
                Err(e) => return Err(UserError::UnhandledError(e.to_string())),
            }
        }
        Ok(result)
    }

    fn get_current_user_account(&self, rest_cfg: &OpenApiConfig) -> Result<UserDetails, UserError> {
        let response = users_current_retrieve(rest_cfg);
        match response {
            Ok(api) => Ok(UserDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_user_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<UserDetails>, UserError> {
        let mut total = vec![];
        let mut service_accounts = self.get_service_account_details(rest_cfg)?;
        let mut user_accounts = self.get_user_account_details(rest_cfg)?;
        total.append(&mut service_accounts); // service accounts have more info, so keep these
                                             // filter out the user accounts that are direct correlations to the service accounts
        user_accounts.retain(|x| x.account_type != "service");
        total.append(&mut user_accounts);

        total.sort_by(|l, r| l.name.to_lowercase().cmp(&r.name.to_lowercase()));
        Ok(total)
    }

    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        user_name: &str,
    ) -> Result<Option<UserDetails>, UserError> {
        // start by looking for the service account, since we have more info
        let mut result = None;

        let mut service_accounts = self.get_service_account_details(rest_cfg)?;
        service_accounts.retain(|x| x.name == user_name);
        if !service_accounts.is_empty() {
            result = Some(service_accounts[0].clone());
        } else {
            let mut user_accounts = self.get_user_account_details(rest_cfg)?;
            user_accounts.retain(|x| x.name == user_name);
            if !user_accounts.is_empty() {
                result = Some(user_accounts[0].clone());
            }
        }

        Ok(result)
    }

    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        user_name: &str,
    ) -> Result<Option<String>, UserError> {
        let details = self.get_details_by_name(rest_cfg, user_name)?;
        Ok(details.map(|d| d.id))
    }

    fn create_service_account(
        &self,
        rest_cfg: &OpenApiConfig,
        user_name: &str,
        description: Option<&str>,
    ) -> Result<UserDetails, UserError> {
        let user_create = ServiceAccountCreateRequest {
            name: user_name.to_string(),
            description: description.map(String::from),
        };
        let response = serviceaccounts_create(rest_cfg, user_create);
        match response {
            Ok(api) => Ok(UserDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    fn update_service_account(
        &self,
        rest_cfg: &OpenApiConfig,
        user_id: &str,
        description: Option<&str>,
    ) -> Result<UserDetails, UserError> {
        let user_update = PatchedServiceAccount {
            url: None,
            id: None,
            user: None,
            description: description.map(String::from),
            created_at: None,
            modified_at: None,
            last_used_at: None,
        };
        let response = serviceaccounts_partial_update(rest_cfg, user_id, Some(user_update));
        match response {
            Ok(api) => Ok(UserDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    fn create_membership(
        &self,
        rest_cfg: &OpenApiConfig,
        user_url: &str,
        role: RoleEnum,
    ) -> Result<(), UserError> {
        let member_create = MembershipCreate {
            user: String::from(user_url),
            role: Some(Box::new(role)),
        };
        let response = memberships_create(rest_cfg, member_create);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    fn update_membership(
        &self,
        rest_cfg: &OpenApiConfig,
        member_id: &str,
        role: Option<RoleEnum>,
    ) -> Result<(), UserError> {
        let member_update = PatchedMembership {
            url: None,
            id: None,
            user: None,
            organization: None,
            role: role.map(Box::new),
            created_at: None,
            modified_at: None,
        };
        let response = memberships_partial_update(rest_cfg, member_id, Some(member_update));
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    pub fn create_user(
        &self,
        rest_cfg: &OpenApiConfig,
        user_name: &str,
        role: &str,
        description: Option<&str>,
    ) -> Result<UserDetails, UserError> {
        let role_enum = to_role_enum(role)?;
        let details = self.create_service_account(rest_cfg, user_name, description)?;

        // must add membership for the account to show up in the list
        let response = self.create_membership(rest_cfg, &details.user_url, role_enum);
        match response {
            Ok(_) => Ok(details),
            Err(e) => {
                self.delete_user(rest_cfg, &details.id)?;
                Err(e)
            }
        }
    }

    pub fn update_user(
        &self,
        rest_cfg: &OpenApiConfig,
        user_id: &str,
        role: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), UserError> {
        let role_enum = role.map(|x| to_role_enum(x).unwrap());
        // do the update anyway, since we need the user_url
        let details = self.update_service_account(rest_cfg, user_id, description)?;
        if role_enum.is_some() {
            self.update_membership(rest_cfg, &details.membership_id, role_enum)?;
        }
        Ok(())
    }

    pub fn delete_user(&self, rest_cfg: &OpenApiConfig, user_id: &str) -> Result<(), UserError> {
        let response = serviceaccounts_destroy(rest_cfg, user_id);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_current_user(&self, rest_cfg: &OpenApiConfig) -> Result<UserDetails, UserError> {
        let details = self.get_current_user_account(rest_cfg)?;
        Ok(details)
    }

    /// Gets a map of user ID to name for all account types.
    pub fn get_user_id_to_name_map(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<UserNameMap, UserError> {
        let mut user_map = UserNameMap::new();
        let mut page_count = 1;
        loop {
            let response = users_list(
                rest_cfg,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(accounts) = data.results {
                        for acct in accounts {
                            user_map.insert(acct.id, acct.name.unwrap_or_default());
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
                Err(e) => return Err(UserError::UnhandledError(e.to_string())),
            }
        }
        Ok(user_map)
    }

    /// Gets a map of user URL to name for all account types.
    pub fn get_user_url_to_name_map(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<UserNameMap, UserError> {
        let mut user_map = UserNameMap::new();
        let mut page_count = 1;
        loop {
            let response = users_list(
                rest_cfg,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(accounts) = data.results {
                        for acct in accounts {
                            user_map.insert(acct.url, acct.name.unwrap_or_default());
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
                Err(e) => return Err(UserError::UnhandledError(e.to_string())),
            }
        }
        Ok(user_map)
    }
}
