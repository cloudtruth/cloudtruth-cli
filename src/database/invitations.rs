use crate::database::{
    auth_details, page_size, response_message, InvitationDetails, InvitationError, OpenApiConfig,
    Users, NO_PAGE_COUNT, NO_PAGE_SIZE,
};
use cloudtruth_restapi::apis::invitations_api::{
    invitations_create, invitations_destroy, invitations_list, invitations_partial_update,
    invitations_resend_create,
};
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{InvitationCreate, PatchedInvitation, RoleEnum};

pub struct Invitations {}

const NO_ORDERING: Option<&str> = None;

fn auth_error(content: &str) -> InvitationError {
    InvitationError::Authentication(auth_details(content))
}

fn response_error(status: &reqwest::StatusCode, content: &str) -> InvitationError {
    match status.as_u16() {
        401 => auth_error(content),
        403 => auth_error(content),
        _ => InvitationError::ResponseError(response_message(status, content)),
    }
}

fn to_role_enum(value: &str) -> Result<RoleEnum, InvitationError> {
    match value.to_uppercase().as_str() {
        "OWNER" => Ok(RoleEnum::OWNER),
        "ADMIN" => Ok(RoleEnum::ADMIN),
        "CONTRIB" => Ok(RoleEnum::CONTRIB),
        "VIEWER" => Ok(RoleEnum::VIEWER),
        _ => Err(InvitationError::InvalidRole(value.to_string())),
    }
}

impl Invitations {
    pub fn new() -> Self {
        Self {}
    }

    fn get_unresolved_invitations(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<InvitationDetails>, InvitationError> {
        let mut result: Vec<InvitationDetails> = Vec::new();
        let mut page_count = 1;
        loop {
            let response = invitations_list(
                rest_cfg,
                None,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                None,
                None,
            );
            match response {
                Ok(data) => {
                    if let Some(invites) = data.results {
                        for inv in invites {
                            result.push(InvitationDetails::from(&inv));
                        }
                        page_count += 1;
                    } else {
                        break;
                    }
                    if data.next.is_none() {
                        break;
                    }
                }
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content))
                }
                Err(e) => return Err(InvitationError::UnhandledError(e.to_string())),
            }
        }
        Ok(result)
    }

    pub fn get_invitation_details(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<Vec<InvitationDetails>, InvitationError> {
        let mut details = self.get_unresolved_invitations(rest_cfg)?;
        self.resolve_user_urls(rest_cfg, &mut details);
        Ok(details)
    }

    pub fn get_details_by_email(
        &self,
        rest_cfg: &OpenApiConfig,
        email: &str,
    ) -> Result<Option<InvitationDetails>, InvitationError> {
        let response = invitations_list(
            rest_cfg,
            Some(email),
            NO_ORDERING,
            NO_PAGE_COUNT,
            NO_PAGE_SIZE,
            None,
            None,
        );
        match response {
            Ok(data) => match data.results {
                Some(list) => match list.is_empty() {
                    true => Ok(None),
                    // TODO: need to resolve the inviter_url??
                    false => Ok(Some(InvitationDetails::from(&list[0]))),
                },
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(InvitationError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        email: &str,
    ) -> Result<Option<String>, InvitationError> {
        let details = self.get_details_by_email(rest_cfg, email)?;
        Ok(details.map(|d| d.id))
    }

    pub fn create_invitation(
        &self,
        rest_cfg: &OpenApiConfig,
        email: &str,
        role: &str,
    ) -> Result<InvitationDetails, InvitationError> {
        let role_enum = to_role_enum(role)?;
        let invite_create = InvitationCreate {
            email: email.to_string(),
            role: Some(Box::new(role_enum)),
        };
        let response = invitations_create(rest_cfg, invite_create);
        match response {
            Ok(api) => Ok(InvitationDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(InvitationError::UnhandledError(e.to_string())),
        }
    }

    pub fn update_invitation(
        &self,
        rest_cfg: &OpenApiConfig,
        invite_id: &str,
        role: Option<&str>,
    ) -> Result<InvitationDetails, InvitationError> {
        let role_enum = role.map(|r| to_role_enum(r).unwrap());
        let invite_update = PatchedInvitation {
            url: None,
            id: None,
            email: None,
            role: role_enum.map(Box::new),
            inviter: None,
            inviter_name: None,
            state: None,
            state_detail: None,
            membership: None,
        };
        let response = invitations_partial_update(rest_cfg, invite_id, Some(invite_update));
        match response {
            Ok(api) => Ok(InvitationDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(InvitationError::UnhandledError(e.to_string())),
        }
    }

    pub fn resend_invitation(
        &self,
        rest_cfg: &OpenApiConfig,
        invite_id: &str,
    ) -> Result<InvitationDetails, InvitationError> {
        let response = invitations_resend_create(rest_cfg, invite_id);
        match response {
            Ok(api) => Ok(InvitationDetails::from(&api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(InvitationError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_invitation(
        &self,
        rest_cfg: &OpenApiConfig,
        invite_id: &str,
    ) -> Result<(), InvitationError> {
        let response = invitations_destroy(rest_cfg, invite_id);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(InvitationError::UnhandledError(e.to_string())),
        }
    }

    fn resolve_user_urls(&self, rest_cfg: &OpenApiConfig, invites: &mut [InvitationDetails]) {
        if !invites.is_empty() {
            let users = Users::new();
            let user_map = users.get_user_url_to_name_map(rest_cfg);
            if let Ok(user_map) = user_map {
                let default_invitername = "".to_string();
                for entry in invites {
                    entry.inviter_name = user_map
                        .get(&entry.inviter_url)
                        .unwrap_or(&default_invitername)
                        .clone();
                }
            }
        }
    }
}
