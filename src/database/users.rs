use crate::database::{
    auth_details, response_message, OpenApiConfig, UserDetails, UserError, PAGE_SIZE,
};
use cloudtruth_restapi::apis::memberships_api::{
    memberships_create, memberships_list, memberships_partial_update,
};
use cloudtruth_restapi::apis::serviceaccounts_api::{
    serviceaccounts_create, serviceaccounts_destroy, serviceaccounts_list,
    serviceaccounts_partial_update,
};
use cloudtruth_restapi::apis::users_api::users_list;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    MembershipCreate, PatchedMembership, PatchedServiceAccount, RoleEnum,
    ServiceAccountCreateRequest,
};

const NO_ORDERING: Option<&str> = None;

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
        let response = serviceaccounts_list(rest_cfg, NO_ORDERING, None, PAGE_SIZE);
        match response {
            Ok(data) => {
                let mut list: Vec<UserDetails> = Vec::new();
                if let Some(accounts) = data.results {
                    for acct in accounts {
                        list.push(UserDetails::from(&acct));
                    }
                }
                Ok(list)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    fn get_user_account_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<UserDetails>, UserError> {
        let response = users_list(rest_cfg, NO_ORDERING, None, PAGE_SIZE, None);
        match response {
            Ok(data) => {
                let mut list: Vec<UserDetails> = Vec::new();
                if let Some(accounts) = data.results {
                    for acct in accounts {
                        list.push(UserDetails::from(&acct));
                    }
                }
                Ok(list)
            }
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
            role: Box::new(role),
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

    /// This version of `get_membership_id()` does not seem to work due to the `user` field value
    /// not being found. In the other version of this function, we get all the memberships and do
    /// the filtering ourself.
    fn _get_membership_id(
        &self,
        rest_cfg: &OpenApiConfig,
        user_url: &str,
    ) -> Result<String, UserError> {
        let response =
            memberships_list(rest_cfg, NO_ORDERING, None, PAGE_SIZE, None, Some(user_url));
        match response {
            Ok(data) => match data.results {
                Some(list) => match list.is_empty() {
                    true => Err(UserError::MembershipNotFound()),
                    false => Ok(list[0].id.clone()),
                },
                None => Err(UserError::MembershipNotFound()),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(UserError::UnhandledError(e.to_string())),
        }
    }

    fn get_membership_id(
        &self,
        rest_cfg: &OpenApiConfig,
        user_url: &str,
    ) -> Result<String, UserError> {
        // should be able do the filtering using the user_url, but the server does not like it!
        let response = memberships_list(rest_cfg, NO_ORDERING, None, PAGE_SIZE, None, None);
        match response {
            Ok(data) => match data.results {
                Some(mut list) => match list.is_empty() {
                    true => Err(UserError::MembershipNotFound()),
                    false => {
                        list.retain(|m| m.user == user_url);
                        if !list.is_empty() {
                            Ok(list[0].id.clone())
                        } else {
                            Err(UserError::MembershipNotFound())
                        }
                    }
                },
                None => Err(UserError::MembershipNotFound()),
            },
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
            let member_id = self.get_membership_id(rest_cfg, &details.user_url)?;
            self.update_membership(rest_cfg, &member_id, role_enum)?;
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
}
