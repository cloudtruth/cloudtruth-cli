/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ServiceAccountCreateResponse {
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "user")]
    pub user: Option<Box<crate::models::User>>,
    /// An optional description of the process or system using the service account.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
    /// The most recent date and time the service account was used.  It will be null if the service account has not been used.
    #[serde(rename = "last_used_at")]
    pub last_used_at: Option<String>,
    /// The API Key to use as a Bearer token for the service account.
    #[serde(rename = "apikey")]
    pub apikey: String,
}

impl ServiceAccountCreateResponse {
    pub fn new(
        url: String,
        id: String,
        user: Option<crate::models::User>,
        created_at: String,
        modified_at: Option<String>,
        last_used_at: Option<String>,
        apikey: String,
    ) -> ServiceAccountCreateResponse {
        ServiceAccountCreateResponse {
            url,
            id,
            user: user.map(Box::new),
            description: None,
            created_at,
            modified_at,
            last_used_at,
            apikey,
        }
    }
}