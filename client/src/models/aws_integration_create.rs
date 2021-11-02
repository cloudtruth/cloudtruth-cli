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
pub struct AwsIntegrationCreate {
    /// An optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
    /// This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator.
    #[serde(rename = "aws_external_id", skip_serializing_if = "Option::is_none")]
    pub aws_external_id: Option<String>,
    /// The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator.
    #[serde(rename = "aws_role_name")]
    pub aws_role_name: String,
}

impl AwsIntegrationCreate {
    pub fn new(
        aws_account_id: String,
        aws_enabled_regions: Vec<crate::models::AwsRegionEnum>,
        aws_enabled_services: Vec<crate::models::AwsServiceEnum>,
        aws_role_name: String,
    ) -> AwsIntegrationCreate {
        AwsIntegrationCreate {
            description: None,
            writable: None,
            aws_account_id,
            aws_enabled_regions,
            aws_enabled_services,
            aws_external_id: None,
            aws_role_name,
        }
    }
}
