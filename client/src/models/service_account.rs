/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceAccount {
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "user")]
    pub user: Box<crate::models::User>,
    /// An optional description of the process or system using the service account.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
    /// The most recent date and time the service account was used.
    #[serde(rename = "last_used_at")]
    pub last_used_at: String,
}

impl ServiceAccount {
    pub fn new(
        url: String,
        id: String,
        user: crate::models::User,
        created_at: String,
        modified_at: String,
        last_used_at: String,
    ) -> ServiceAccount {
        ServiceAccount {
            url,
            id,
            user: Box::new(user),
            description: None,
            created_at,
            modified_at,
            last_used_at,
        }
    }
}
