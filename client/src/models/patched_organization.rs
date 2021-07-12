/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your parameters and secrets making them easier to manage and use.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatchedOrganization {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A unique identifier for the organization.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The organization name.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Indicates if this Organization is the one currently targeted by the Bearer token used by the client to authorize.
    #[serde(rename = "current", skip_serializing_if = "Option::is_none")]
    pub current: Option<bool>,
    #[serde(
        rename = "subscription_expires_at",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_expires_at: Option<String>,
    #[serde(rename = "subscription_id", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(
        rename = "subscription_plan_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_plan_id: Option<String>,
    #[serde(
        rename = "subscription_plan_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_plan_name: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl PatchedOrganization {
    pub fn new() -> PatchedOrganization {
        PatchedOrganization {
            url: None,
            id: None,
            name: None,
            current: None,
            subscription_expires_at: None,
            subscription_id: None,
            subscription_plan_id: None,
            subscription_plan_name: None,
            created_at: None,
            modified_at: None,
        }
    }
}
