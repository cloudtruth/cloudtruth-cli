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
pub struct PatchedMembership {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A unique identifier for the membership.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The user of the membership.
    #[serde(rename = "user", skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// The organization that the user is a member of.
    #[serde(rename = "organization", skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// The role that the user has in the organization.
    #[serde(rename = "role", skip_serializing_if = "Option::is_none")]
    pub role: Option<Box<crate::models::RoleEnum>>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl PatchedMembership {
    pub fn new() -> PatchedMembership {
        PatchedMembership {
            url: None,
            id: None,
            user: None,
            organization: None,
            role: None,
            created_at: None,
            modified_at: None,
        }
    }
}
