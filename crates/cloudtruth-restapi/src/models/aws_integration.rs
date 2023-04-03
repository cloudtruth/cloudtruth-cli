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
pub struct AwsIntegration {
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
    /// The AWS Account ID.
    #[serde(rename = "aws_account_id")]
    pub aws_account_id: String,
    /// The AWS regions to integrate with.
    #[serde(rename = "aws_enabled_regions")]
    pub aws_enabled_regions: Vec<crate::models::AwsRegionEnum>,
    /// The AWS services to integrate with.
    #[serde(rename = "aws_enabled_services")]
    pub aws_enabled_services: Vec<crate::models::AwsServiceEnum>,
    /// This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  CloudTruth will generate a random value for you to give to your AWS Administrator in order to create the necessary IAM role for proper access.
    #[serde(rename = "aws_external_id", skip_serializing_if = "Option::is_none")]
    pub aws_external_id: Option<String>,
    /// If present, this is the KMS Key Id that is used to push values.  This key must be accessible in the AWS account (it cannot be an ARN to a key in another AWS account).
    #[serde(rename = "aws_kms_key_id", skip_serializing_if = "Option::is_none")]
    pub aws_kms_key_id: Option<String>,
    /// The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator.
    #[serde(rename = "aws_role_name")]
    pub aws_role_name: String,
}

impl AwsIntegration {
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
        aws_account_id: String,
        aws_enabled_regions: Vec<crate::models::AwsRegionEnum>,
        aws_enabled_services: Vec<crate::models::AwsServiceEnum>,
        aws_role_name: String,
    ) -> AwsIntegration {
        AwsIntegration {
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
            aws_account_id,
            aws_enabled_regions,
            aws_enabled_services,
            aws_external_id: None,
            aws_kms_key_id: None,
            aws_role_name,
        }
    }
}
