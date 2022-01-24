/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// PatchedGitHubPull : Pull actions can be configured to get configuration and secrets from integrations on demand.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PatchedGitHubPull {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Unique identifier for the action.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The action name.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The optional description for the action.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The most recent task run for this action.
    #[serde(rename = "latest_task", skip_serializing_if = "Option::is_none")]
    pub latest_task: Option<Box<crate::models::GitHubPullTask>>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// Allow the pull to create environments.  Any automatically created environments will be children of the `default` environment.  If an environment needs to be created but the action does not allow it, a task step will be added with a null operation, and success_detail will indicate the action did not allow it.
    #[serde(
        rename = "create_environments",
        skip_serializing_if = "Option::is_none"
    )]
    pub create_environments: Option<bool>,
    /// Allow the pull to create projects.  If a project needs to be created but the action does not allow it, a task step will be added with a null operation, and success_detail will indicate the action did not allow it.
    #[serde(rename = "create_projects", skip_serializing_if = "Option::is_none")]
    pub create_projects: Option<bool>,
    /// When set to dry-run mode an action will report the changes that it would have made in task steps, however those changes are not actually performed.
    #[serde(rename = "dry_run", skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Values being managed by a mapped pull.
    #[serde(rename = "mapped_values", skip_serializing_if = "Option::is_none")]
    pub mapped_values: Option<Vec<crate::models::Value>>,
    /// The pull mode used.  A pattern pull uses a pattern-matching resource string with mustache-style markers to identify the project, parameter, and environment names.  A mapped pull uses a specific resource and JMESpath expression to deliver a value to a specific project, parameter, and environment.  This leverages external value linkages made in the value editor, and there is one mapped pull per integration provided by the system so that you can trigger external value pull synchronizations.
    #[serde(rename = "mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<Box<crate::models::ModeEnum>>,
}

impl PatchedGitHubPull {
    /// Pull actions can be configured to get configuration and secrets from integrations on demand.
    pub fn new() -> PatchedGitHubPull {
        PatchedGitHubPull {
            url: None,
            id: None,
            name: None,
            description: None,
            latest_task: None,
            created_at: None,
            modified_at: None,
            create_environments: None,
            create_projects: None,
            dry_run: None,
            mapped_values: None,
            mode: None,
        }
    }
}
