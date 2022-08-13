use cloudtruth_restapi::models::Group;
// use once_cell::sync::OnceCell;

// static DEFAULT_GROUP_VALUE: OnceCell<Group> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct GroupDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub users: Vec<String>,

    pub created_at: String,
    pub modified_at: String,
}

impl GroupDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "users" => self.users.join(", "),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

// fn default_user() -> &'static User {
//     DEFAULT_USER_VALUE.get_or_init(|| User {
//         url: "".to_string(),
//         id: "".to_string(),
//         // NOTE: currently only used from a service account conversion, so guess at account type
//         _type: Some("service?".to_string()),
//         name: Some("Unknown".to_string()),
//         organization_name: None,
//         membership_id: None,
//         role: None,
//         email: None,
//         picture_url: None,
//         created_at: "".to_string(),
//         modified_at: "".to_string(),
//     })
// }

/// Converts from the OpenApi `Group` model to the CloudTruth `GroupDetails`
impl From<&Group> for GroupDetails {
    fn from(api: &Group) -> Self {
        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            name: api.name.clone(),
            description: api.description.clone().unwrap_or_default(),
            users: api.users.clone(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

// #[cfg(test)]
// mod test {
//     use crate::database::UserDetails;
//     use cloudtruth_restapi::models::PaginatedServiceAccountList;

//     #[test]
//     fn service_account_null_user() {
//         let data = r#"{"count":1,"next":null,"previous":null,"results":[{"url":"https://api.staging.cloudtruth.io/api/v1/serviceaccounts/auth0%7C61949e0bc0d28400705bc3cf/","id":"auth0|61949e0bc0d28400705bc3cf","user":null,"description":"Updated description","created_at":"2021-11-17T06:15:41.676634Z","modified_at":"2021-11-17T06:16:00.758835Z","last_used_at":"2021-11-17T06:16:17.334174Z"}]}"#;
//         let result: serde_json::Result<PaginatedServiceAccountList> = serde_json::from_str(data);
//         assert_eq!(false, result.is_err());
//         let list = result.unwrap();
//         assert_eq!(1, list.count.unwrap_or(-1));
//         let items = list.results.unwrap();
//         let entry = &items[0];
//         assert_eq!(entry.user, None);
//         assert_eq!(
//             entry.last_used_at,
//             Some("2021-11-17T06:16:17.334174Z".to_string())
//         );

//         let details = UserDetails::from(entry);
//         assert_eq!(details.id, entry.id);
//         assert_eq!(details.name, "Unknown");
//         assert_eq!(details.account_type, "service?");
//         assert_eq!(details.email, "");
//         assert_eq!(details.user_url, "");
//         assert_eq!(details.last_used, entry.last_used_at.clone().unwrap());
//     }

//     #[test]
//     fn service_account_null_last_used_by() {
//         let data = r#"{"count":1,"next":null,"previous":null,"results":[{"url":"https://api.staging.cloudtruth.io/api/v1/serviceaccounts/auth0%7C61949e36c0baff006a6d1aea/","id":"auth0|61949e36c0baff006a6d1aea","user":{"url":"https://api.staging.cloudtruth.io/api/v1/users/auth0%7C61949e36c0baff006a6d1aea/","id":"auth0|61949e36c0baff006a6d1aea","type":"service","name":"test-user-name-Linux-37","email":"serviceaccount+ajwflw8wzpc1quhp@cloudtruth.com","picture_url":null,"created_at":"2021-11-17T06:16:24.082794Z","modified_at":"2021-11-17T06:16:24.548713Z"},"description":"Description on create","created_at":"2021-11-17T06:16:24.558748Z","modified_at":"2021-11-17T06:16:24.558748Z","last_used_at":null}]}"#;
//         let result: serde_json::Result<PaginatedServiceAccountList> = serde_json::from_str(data);
//         assert_eq!(false, result.is_err());
//         let list = result.unwrap();
//         assert_eq!(1, list.count.unwrap_or(-1));
//         let items = list.results.unwrap();
//         let entry = &items[0];
//         assert_ne!(entry.user, None);
//         assert_eq!(entry.last_used_at, None);

//         let details = UserDetails::from(entry);
//         assert_eq!(details.id, entry.id);
//         assert_eq!(details.name, "test-user-name-Linux-37");
//         assert_eq!(details.account_type, "service");
//         assert_eq!(
//             details.email,
//             "serviceaccount+ajwflw8wzpc1quhp@cloudtruth.com"
//         );
//         assert_eq!(
//             details.user_url,
//             "https://api.staging.cloudtruth.io/api/v1/users/auth0%7C61949e36c0baff006a6d1aea/"
//         );
//         assert_eq!(details.last_used, "");
//     }
// }
