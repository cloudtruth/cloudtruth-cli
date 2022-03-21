/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// AwsPull : Pull actions can be configured to get configuration and secrets from integrations on demand.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AwsPull {
    #[serde(rename = "url")]
    pub url: String,
    /// Unique identifier for the action.
    #[serde(rename = "id")]
    pub id: String,
    /// The action name.
    #[serde(rename = "name")]
    pub name: String,
    /// The optional description for the action.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The most recent task run for this action.
    #[serde(rename = "latest_task")]
    pub latest_task: Option<Box<crate::models::AwsPullTask>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
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
    #[serde(rename = "mapped_values")]
    pub mapped_values: Vec<crate::models::Value>,
    /// The pull mode used.  A pattern pull uses a pattern-matching resource string with mustache-style markers to identify the project, parameter, and environment names, or with a Python regular expression that uses named capture groups that define the same three concepts.  A mapped pull uses a specific resource and JMESpath expression to deliver a value to a specific project, parameter, and environment.  This leverages external value linkages made in the value editor, and there is one mapped pull per integration provided by the system so that you can trigger external value pull synchronizations.
    #[serde(rename = "mode")]
    pub mode: Option<Box<crate::models::ModeEnum>>,
    /// The AWS region to use.  This region must be enabled in the integration.
    #[serde(rename = "region")]
    pub region: Option<Box<crate::models::AwsRegionEnum>>,
    /// The AWS service to use.  This service must be enabled in the integration.
    #[serde(rename = "service")]
    pub service: Option<Box<crate::models::AwsServiceEnum>>,
    /// Defines a pattern matching string that contains either mustache or regular expression syntax (with named capture groups) that locate the environment, project, and parameter name of the content you are looking for.  If you are using mustache pattern matching, use:    - ``{{ environment }}`` to identify the environment name   - ``{{ parameter }}`` to identify the parameter name   - ``{{ project }}`` to identify the project name  If you are using a regular expression, use Python syntax with named capture groups that locate the `environment`, `project`, and `parameter`.
    #[serde(rename = "resource")]
    pub resource: Option<String>,
}

impl AwsPull {
    /// Pull actions can be configured to get configuration and secrets from integrations on demand.
    pub fn new(
        url: String,
        id: String,
        name: String,
        latest_task: Option<crate::models::AwsPullTask>,
        created_at: String,
        modified_at: String,
        mapped_values: Vec<crate::models::Value>,
        mode: Option<crate::models::ModeEnum>,
        region: Option<crate::models::AwsRegionEnum>,
        service: Option<crate::models::AwsServiceEnum>,
        resource: Option<String>,
    ) -> AwsPull {
        AwsPull {
            url,
            id,
            name,
            description: None,
            latest_task: latest_task.map(Box::new),
            created_at,
            modified_at,
            create_environments: None,
            create_projects: None,
            dry_run: None,
            mapped_values,
            mode: mode.map(Box::new),
            region: region.map(Box::new),
            service: service.map(Box::new),
            resource,
        }
    }
}
