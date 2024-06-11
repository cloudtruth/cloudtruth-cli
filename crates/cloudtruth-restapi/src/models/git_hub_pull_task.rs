/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// GitHubPullTask : Pull task for a GitHub integration.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GitHubPullTask {
    #[serde(rename = "url")]
    pub url: String,
    /// The unique identifier for the task.
    #[serde(rename = "id")]
    pub id: String,
    /// The reason why this task was triggered.
    #[serde(rename = "reason", skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Indicates task steps were only simulated, not actually performed.
    #[serde(rename = "dry_run", skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// The current state of this task.
    #[serde(rename = "state", skip_serializing_if = "Option::is_none")]
    pub state: Option<Box<crate::models::StateEnum>>,
    /// If an error occurs early during processing, before attempting to process values, this code may be helpful in determining the problem.
    #[serde(rename = "error_code", skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// If an error occurs early during processing, before attempting to process values, this detail may be helpful in determining the problem.
    #[serde(rename = "error_detail", skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
}

impl GitHubPullTask {
    /// Pull task for a GitHub integration.
    pub fn new(
        url: String,
        id: String,
        created_at: String,
        modified_at: Option<String>,
    ) -> GitHubPullTask {
        GitHubPullTask {
            url,
            id,
            reason: None,
            dry_run: None,
            state: None,
            error_code: None,
            error_detail: None,
            created_at,
            modified_at,
        }
    }
}
