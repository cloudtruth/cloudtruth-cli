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
pub struct PatchedAzureKeyVaultIntegration {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The unique identifier for the integration.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// An optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified.
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Box<crate::models::StatusEnum>>,
    /// If an error occurs, more details will be available in this field.
    #[serde(rename = "status_detail", skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    /// The last time the status was evaluated.
    #[serde(
        rename = "status_last_checked_at",
        skip_serializing_if = "Option::is_none"
    )]
    pub status_last_checked_at: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(rename = "fqn", skip_serializing_if = "Option::is_none")]
    pub fqn: Option<String>,
    /// The type of integration.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
    /// Allow actions to write to the integration.
    #[serde(rename = "writable", skip_serializing_if = "Option::is_none")]
    pub writable: Option<bool>,
    /// The Azure Key Vault name.
    #[serde(rename = "vault_name", skip_serializing_if = "Option::is_none")]
    pub vault_name: Option<String>,
    /// The Azure Tenant ID.
    #[serde(rename = "tenant_id", skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl PatchedAzureKeyVaultIntegration {
    pub fn new() -> PatchedAzureKeyVaultIntegration {
        PatchedAzureKeyVaultIntegration {
            url: None,
            id: None,
            name: None,
            description: None,
            status: None,
            status_detail: None,
            status_last_checked_at: None,
            created_at: None,
            modified_at: None,
            fqn: None,
            _type: None,
            writable: None,
            vault_name: None,
            tenant_id: None,
        }
    }
}
