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
pub struct AzureKeyVaultIntegration {
    #[serde(rename = "url")]
    pub url: String,
    /// The unique identifier for the integration.
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    /// An optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified.
    #[serde(rename = "status")]
    pub status: Option<Box<crate::models::StatusEnum>>,
    /// If an error occurs, more details will be available in this field.
    #[serde(rename = "status_detail")]
    pub status_detail: String,
    /// The last time the status was evaluated.
    #[serde(rename = "status_last_checked_at")]
    pub status_last_checked_at: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
    #[serde(rename = "fqn")]
    pub fqn: String,
    /// The type of integration.
    #[serde(rename = "type")]
    pub _type: String,
    /// Allow actions to write to the integration.
    #[serde(rename = "writable", skip_serializing_if = "Option::is_none")]
    pub writable: Option<bool>,
    /// The Azure Key Vault name.
    #[serde(rename = "vault_name")]
    pub vault_name: String,
    /// The Azure Tenant ID.
    #[serde(rename = "tenant_id")]
    pub tenant_id: String,
}

impl AzureKeyVaultIntegration {
    pub fn new(
        url: String,
        id: String,
        name: String,
        status: Option<crate::models::StatusEnum>,
        status_detail: String,
        status_last_checked_at: Option<String>,
        created_at: String,
        modified_at: Option<String>,
        fqn: String,
        _type: String,
        vault_name: String,
        tenant_id: String,
    ) -> AzureKeyVaultIntegration {
        AzureKeyVaultIntegration {
            url,
            id,
            name,
            description: None,
            status: status.map(Box::new),
            status_detail,
            status_last_checked_at,
            created_at,
            modified_at,
            fqn,
            _type,
            writable: None,
            vault_name,
            tenant_id,
        }
    }
}
