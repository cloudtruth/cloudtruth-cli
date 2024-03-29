/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// PatchedAzureKeyVaultPushUpdate : Update a push.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PatchedAzureKeyVaultPushUpdate {
    /// The action name.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The optional description for the action.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Projects that are included in the push.
    #[serde(rename = "projects", skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    /// Tags are used to select parameters by environment from the projects included in the push.  You cannot have two tags from the same environment in the same push.
    #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Defines a path through the integration to the location where values will be pushed.  The following mustache-style substitutions can be used in the string:    - ``{{ environment }}`` to insert the environment name   - ``{{ parameter }}`` to insert the parameter name   - ``{{ project }}`` to insert the project name   - ``{{ push }}`` to insert the push name   - ``{{ tag }}`` to insert the tag name  We recommend that you use project, environment, and parameter at a minimum to disambiguate your pushed resource identifiers.  If you include multiple projects in the push, the `project` substitution is required.  If you include multiple tags from different environments in the push, the `environment` substitution is required.  If you include multiple tags from the same environment in the push, the `tag` substitution is required.  In all cases, the `parameter` substitution is always required.
    #[serde(rename = "resource", skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    /// When set to dry-run mode an action will report the changes that it would have made in task steps, however those changes are not actually performed.
    #[serde(rename = "dry_run", skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Normally, push will check to see if it originated the values in the destination before making changes to them.  Forcing a push disables the ownership check.
    #[serde(rename = "force", skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Normally, push will process all parameters including those that flow in from project dependencies.  Declaring a push as `local` will cause it to only process the parameters defined in the selected projects.
    #[serde(rename = "local", skip_serializing_if = "Option::is_none")]
    pub local: Option<bool>,
    /// This setting allows parameters (non-secrets) to be pushed to a destination that only supports storing secrets.  This may increase your overall cost from the cloud provider as some cloud providers charge a premium for secrets-only storage.
    #[serde(rename = "coerce_parameters", skip_serializing_if = "Option::is_none")]
    pub coerce_parameters: Option<bool>,
    /// Include parameters (non-secrets) in the values being pushed.  This setting requires the destination to support parameters or for the `coerce_parameters` flag to be enabled, otherwise the push will fail.
    #[serde(rename = "include_parameters", skip_serializing_if = "Option::is_none")]
    pub include_parameters: Option<bool>,
    /// Include secrets in the values being pushed.  This setting requires the destination to support secrets, otherwise the push will fail.
    #[serde(rename = "include_secrets", skip_serializing_if = "Option::is_none")]
    pub include_secrets: Option<bool>,
}

impl PatchedAzureKeyVaultPushUpdate {
    /// Update a push.
    pub fn new() -> PatchedAzureKeyVaultPushUpdate {
        PatchedAzureKeyVaultPushUpdate {
            name: None,
            description: None,
            projects: None,
            tags: None,
            resource: None,
            dry_run: None,
            force: None,
            local: None,
            coerce_parameters: None,
            include_parameters: None,
            include_secrets: None,
        }
    }
}
