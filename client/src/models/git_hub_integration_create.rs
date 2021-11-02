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
pub struct GitHubIntegrationCreate {
    /// An optional description for the integration.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allow actions to write to the integration.
    #[serde(rename = "writable", skip_serializing_if = "Option::is_none")]
    pub writable: Option<bool>,
    #[serde(rename = "gh_installation_id")]
    pub gh_installation_id: i32,
}

impl GitHubIntegrationCreate {
    pub fn new(gh_installation_id: i32) -> GitHubIntegrationCreate {
        GitHubIntegrationCreate {
            description: None,
            writable: None,
            gh_installation_id,
        }
    }
}
