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
pub struct AwsIntegration {
    #[serde(rename = "url")]
    pub url: String,
    /// The unique identifier for the integration.
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    /// The optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified.
    #[serde(rename = "status")]
    pub status: String,
    /// If an error occurs, more details will be available in this field.
    #[serde(rename = "status_detail")]
    pub status_detail: String,
    /// The last time the status was evaluated.
    #[serde(rename = "status_last_checked_at")]
    pub status_last_checked_at: String,
    /// The type of integration.
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
    #[serde(rename = "fqn")]
    pub fqn: String,
    /// The AWS Account ID.
    #[serde(rename = "aws_account_id")]
    pub aws_account_id: String,
    /// The AWS regions to integrate with.
    #[serde(rename = "aws_enabled_regions")]
    pub aws_enabled_regions: Vec<crate::models::AwsEnabledRegionsEnum>,
    /// The AWS services to integrate with.
    #[serde(rename = "aws_enabled_services")]
    pub aws_enabled_services: Vec<crate::models::AwsEnabledServicesEnum>,
    /// This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator.
    #[serde(rename = "aws_external_id", skip_serializing_if = "Option::is_none")]
    pub aws_external_id: Option<String>,
    /// The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator.
    #[serde(rename = "aws_role_name")]
    pub aws_role_name: String,
}

impl AwsIntegration {
    pub fn new(
        url: String,
        id: String,
        name: String,
        status: String,
        status_detail: String,
        status_last_checked_at: String,
        _type: String,
        created_at: String,
        modified_at: String,
        fqn: String,
        aws_account_id: String,
        aws_enabled_regions: Vec<crate::models::AwsEnabledRegionsEnum>,
        aws_enabled_services: Vec<crate::models::AwsEnabledServicesEnum>,
        aws_role_name: String,
    ) -> AwsIntegration {
        AwsIntegration {
            url,
            id,
            name,
            description: None,
            status,
            status_detail,
            status_last_checked_at,
            _type,
            created_at,
            modified_at,
            fqn,
            aws_account_id,
            aws_enabled_regions,
            aws_enabled_services,
            aws_external_id: None,
            aws_role_name,
        }
    }
}
