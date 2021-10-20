use cloudtruth_restapi::models::{ServiceAccount, ServiceAccountCreateResponse, User};

/// This is to provide a unified view of users that includes `ServiceAccount` and `User` properties.
#[derive(Clone, Debug)]
pub struct UserDetails {
    pub id: String,
    pub user_url: String,
    pub name: String,
    pub account_type: String,
    pub email: String,

    pub created_at: String,
    pub modified_at: String,

    // these come from the service account
    pub last_used: String,
    pub description: String,
    pub api_key: String, // only populated for create
}

impl UserDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.name.clone(),
            "type" => self.account_type.clone(),
            "email" => self.email.clone(),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),

            "last-used" => self.last_used.clone(),
            "description" => self.description.clone(),

            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

fn extract_name(user: &User) -> String {
    user.name
        .clone()
        .unwrap_or_else(|| user.email.clone().unwrap_or_else(|| user.id.clone()))
}

/// Converts from the OpenApi `User` model to the CloudTruth `UserDetails`
impl From<&User> for UserDetails {
    fn from(api: &User) -> Self {
        Self {
            id: api.id.clone(),
            user_url: api.url.clone(),
            name: extract_name(api),
            email: api.email.clone().unwrap_or_default(),
            account_type: api._type.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),

            // not used for a user account
            last_used: "".to_string(),
            description: "".to_string(),
            api_key: "".to_string(),
        }
    }
}

/// Converts from the OpenApi `ServiceAccount` model to the CloudTruth `UserDetails`
impl From<&ServiceAccount> for UserDetails {
    fn from(api: &ServiceAccount) -> Self {
        let user = &*api.user;
        Self {
            id: api.id.clone(),
            user_url: user.url.clone(),
            name: extract_name(user),
            email: user.email.clone().unwrap_or_default(),
            account_type: user._type.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),

            last_used: api.last_used_at.clone().unwrap_or_default(),
            description: api.description.clone().unwrap_or_default(),
            api_key: "".to_string(),
        }
    }
}

impl From<&ServiceAccountCreateResponse> for UserDetails {
    fn from(api: &ServiceAccountCreateResponse) -> Self {
        let user = &*api.user;
        Self {
            id: api.id.clone(),
            user_url: user.url.clone(),
            name: extract_name(user),
            account_type: user._type.clone().unwrap_or_default(),
            email: user.email.clone().unwrap_or_default(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
            last_used: api.last_used_at.clone().unwrap_or_default(),
            description: api.description.clone().unwrap_or_default(),
            api_key: api.apikey.clone(),
        }
    }
}
