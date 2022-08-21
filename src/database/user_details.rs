use cloudtruth_restapi::models::{ServiceAccount, ServiceAccountCreateResponse, User};
use once_cell::sync::OnceCell;

static DEFAULT_USER_VALUE: OnceCell<User> = OnceCell::new();

/// This is to provide a unified view of users that includes `ServiceAccount` and `User` properties.
#[derive(Clone, Debug)]
pub struct UserDetails {
    pub id: String,
    pub user_url: String,
    pub name: String,
    pub account_type: String,
    pub email: String,
    pub organization: String,
    pub membership_id: String,
    pub role: String,

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
            "email" => match self.account_type.as_str() {
                "service" => "".to_string(),
                _ => self.email.clone(),
            },
            "organization" => self.organization.clone(),
            "role" => self.role.clone(),
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

fn default_user() -> &'static User {
    DEFAULT_USER_VALUE.get_or_init(|| User {
        url: "".to_string(),
        id: "".to_string(),
        // NOTE: currently only used from a service account conversion, so guess at account type
        _type: Some("service?".to_string()),
        name: Some("Unknown".to_string()),
        organization_name: None,
        membership_id: None,
        role: None,
        email: None,
        picture_url: None,
        created_at: "".to_string(),
        modified_at: "".to_string(),
    })
}

/// Converts from the OpenApi `User` model to the CloudTruth `UserDetails`
impl From<&User> for UserDetails {
    fn from(api: &User) -> Self {
        Self {
            id: api.id.clone(),
            user_url: api.url.clone(),
            name: extract_name(api),
            email: api.email.clone().unwrap_or_default(),
            organization: api.organization_name.clone().unwrap_or_default(),
            membership_id: api.membership_id.clone().unwrap_or_default(),
            role: api.role.clone().unwrap_or_default().to_lowercase(),
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
        let user: &User;
        if let Some(ref uref) = api.user {
            user = uref;
        } else {
            user = default_user();
        };
        Self {
            id: api.id.clone(),
            user_url: user.url.clone(),
            name: extract_name(user),
            email: user.email.clone().unwrap_or_default(),
            organization: user.organization_name.clone().unwrap_or_default(),
            membership_id: user.membership_id.clone().unwrap_or_default(),
            role: user.role.clone().unwrap_or_default().to_lowercase(),
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
        let user: &User;
        if let Some(ref uref) = api.user {
            user = uref;
        } else {
            user = default_user();
        };
        Self {
            id: api.id.clone(),
            user_url: user.url.clone(),
            name: extract_name(user),
            account_type: user._type.clone().unwrap_or_default(),
            email: user.email.clone().unwrap_or_default(),
            organization: user.organization_name.clone().unwrap_or_default(),
            membership_id: user.membership_id.clone().unwrap_or_default(),
            role: user.role.clone().unwrap_or_default().to_lowercase(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
            last_used: api.last_used_at.clone().unwrap_or_default(),
            description: api.description.clone().unwrap_or_default(),
            api_key: api.apikey.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::database::UserDetails;
    use cloudtruth_restapi::models::PaginatedServiceAccountList;

    #[test]
    fn service_account_null_user() {
        let data = r#"{"count":1,"next":null,"previous":null,"results":[{"url":"https://api.staging.cloudtruth.io/api/v1/serviceaccounts/auth0%7C61949e0bc0d28400705bc3cf/","id":"auth0|61949e0bc0d28400705bc3cf","user":null,"description":"Updated description","created_at":"2021-11-17T06:15:41.676634Z","modified_at":"2021-11-17T06:16:00.758835Z","last_used_at":"2021-11-17T06:16:17.334174Z"}]}"#;
        let result: serde_json::Result<PaginatedServiceAccountList> = serde_json::from_str(data);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(1, list.count.unwrap_or(-1));
        let items = list.results.unwrap();
        let entry = &items[0];
        assert_eq!(entry.user, None);
        assert_eq!(
            entry.last_used_at,
            Some("2021-11-17T06:16:17.334174Z".to_string())
        );

        let details = UserDetails::from(entry);
        assert_eq!(details.id, entry.id);
        assert_eq!(details.name, "Unknown");
        assert_eq!(details.account_type, "service?");
        assert_eq!(details.email, "");
        assert_eq!(details.user_url, "");
        assert_eq!(details.last_used, entry.last_used_at.clone().unwrap());
    }

    #[test]
    fn service_account_null_last_used_by() {
        let data = r#"{"count":1,"next":null,"previous":null,"results":[{"url":"https://api.staging.cloudtruth.io/api/v1/serviceaccounts/auth0%7C61949e36c0baff006a6d1aea/","id":"auth0|61949e36c0baff006a6d1aea","user":{"url":"https://api.staging.cloudtruth.io/api/v1/users/auth0%7C61949e36c0baff006a6d1aea/","id":"auth0|61949e36c0baff006a6d1aea","type":"service","name":"test-user-name-Linux-37","email":"serviceaccount+ajwflw8wzpc1quhp@cloudtruth.com","picture_url":null,"created_at":"2021-11-17T06:16:24.082794Z","modified_at":"2021-11-17T06:16:24.548713Z"},"description":"Description on create","created_at":"2021-11-17T06:16:24.558748Z","modified_at":"2021-11-17T06:16:24.558748Z","last_used_at":null}]}"#;
        let result: serde_json::Result<PaginatedServiceAccountList> = serde_json::from_str(data);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(1, list.count.unwrap_or(-1));
        let items = list.results.unwrap();
        let entry = &items[0];
        assert_ne!(entry.user, None);
        assert_eq!(entry.last_used_at, None);

        let details = UserDetails::from(entry);
        assert_eq!(details.id, entry.id);
        assert_eq!(details.name, "test-user-name-Linux-37");
        assert_eq!(details.account_type, "service");
        assert_eq!(
            details.email,
            "serviceaccount+ajwflw8wzpc1quhp@cloudtruth.com"
        );
        assert_eq!(
            details.user_url,
            "https://api.staging.cloudtruth.io/api/v1/users/auth0%7C61949e36c0baff006a6d1aea/"
        );
        assert_eq!(details.last_used, "");
    }
}
