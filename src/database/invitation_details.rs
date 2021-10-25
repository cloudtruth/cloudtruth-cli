use cloudtruth_restapi::models::Invitation;

/// This is to provide a unified view of users that includes `ServiceAccount` and `User` properties.
#[derive(Clone, Debug)]
pub struct InvitationDetails {
    pub id: String,
    pub url: String,
    pub email: String,
    pub role: String,
    pub inviter_url: String,
    pub inviter_name: String,
    pub state: String,
    pub state_detail: String,
    pub membership: String,
}

impl From<&Invitation> for InvitationDetails {
    fn from(api: &Invitation) -> Self {
        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            email: api.email.clone(),
            role: api.role.to_string().to_lowercase(),
            inviter_url: api.inviter.clone(),
            inviter_name: "".to_string(), // need to fill in later
            state: api.state.to_lowercase(),
            state_detail: api.state_detail.clone(),
            membership: api.membership.clone().unwrap_or_default(),
        }
    }
}

impl InvitationDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "email" => self.email.clone(),
            "role" => self.role.clone(),
            "inviter-url" => self.inviter_url.clone(),
            "inviter-name" => self.inviter_name.clone(),
            "state" => self.state.clone(),
            "state-detail" => self.state_detail.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}
