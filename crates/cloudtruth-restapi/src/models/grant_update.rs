/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GrantUpdate {
    #[serde(rename = "url")]
    pub url: String,
    /// A unique identifier for the grant.
    #[serde(rename = "id")]
    pub id: String,
    /// The URI of a principal for the grant; this must reference a user or group.
    #[serde(rename = "principal")]
    pub principal: String,
    /// The URI of a scope for the grant; this must reference a project or environment.
    #[serde(rename = "scope")]
    pub scope: String,
    /// The role that the principal has in the given scope.
    #[serde(rename = "role")]
    pub role: Option<Box<crate::models::RoleEnum>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
}

impl GrantUpdate {
    pub fn new(
        url: String,
        id: String,
        principal: String,
        scope: String,
        role: Option<crate::models::RoleEnum>,
        created_at: String,
        modified_at: Option<String>,
    ) -> GrantUpdate {
        GrantUpdate {
            url,
            id,
            principal,
            scope,
            role: role.map(Box::new),
            created_at,
            modified_at,
        }
    }
}
