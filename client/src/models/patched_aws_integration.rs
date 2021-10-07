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
pub struct PatchedAwsIntegration {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The unique identifier for the integration.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified.
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// If an error occurs, more details will be available in this field.
    #[serde(rename = "status_detail", skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    /// The last time the status was evaluated.
    #[serde(
        rename = "status_last_checked_at",
        skip_serializing_if = "Option::is_none"
    )]
    pub status_last_checked_at: Option<String>,
    /// The type of integration.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// The actions to allow.
    #[serde(rename = "allow", skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<crate::models::AllowEnum>>,
    #[serde(rename = "fqn", skip_serializing_if = "Option::is_none")]
    pub fqn: Option<String>,
    /// The AWS Account ID.
    #[serde(rename = "aws_account_id", skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,
    /// The AWS regions to integrate with.
    #[serde(
        rename = "aws_enabled_regions",
        skip_serializing_if = "Option::is_none"
    )]
    pub aws_enabled_regions: Option<Vec<crate::models::AwsRegionEnum>>,
    /// The AWS services to integrate with.
    #[serde(
        rename = "aws_enabled_services",
        skip_serializing_if = "Option::is_none"
    )]
    pub aws_enabled_services: Option<Vec<crate::models::AwsServiceEnum>>,
    /// This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator.
    #[serde(rename = "aws_external_id", skip_serializing_if = "Option::is_none")]
    pub aws_external_id: Option<String>,
    /// The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator.
    #[serde(rename = "aws_role_name", skip_serializing_if = "Option::is_none")]
    pub aws_role_name: Option<String>,
}

impl PatchedAwsIntegration {
    pub fn new() -> PatchedAwsIntegration {
        PatchedAwsIntegration {
            url: None,
            id: None,
            name: None,
            description: None,
            status: None,
            status_detail: None,
            status_last_checked_at: None,
            _type: None,
            created_at: None,
            modified_at: None,
            allow: None,
            fqn: None,
            aws_account_id: None,
            aws_enabled_regions: None,
            aws_enabled_services: None,
            aws_external_id: None,
            aws_role_name: None,
        }
    }
}
