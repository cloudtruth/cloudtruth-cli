/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// GitHubPullTaskStep : Pull task step for a GitHub integration.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GitHubPullTaskStep {
    #[serde(rename = "url")]
    pub url: String,
    /// Unique identifier for a task step.
    #[serde(rename = "id")]
    pub id: String,
    /// The operation performed, if any.  When the operation is an update, there may be additional details in the success_detail field to describe the change.  When the project is filled in but the environment and parameterare not, the operation is on the project.  When the environmentis filled in but the project and parameter are not, the operationis on the environment.  When the project and parameter are filledin but the environment is not, the operation is on the parameter.When all three are filled in, the operation is on the value ofthe parameter of the project in the specified environment.
    #[serde(rename = "operation", skip_serializing_if = "Option::is_none")]
    pub operation: Option<Box<crate::models::OperationEnum>>,
    /// Indicates if the operation was successful.
    #[serde(rename = "success")]
    pub success: bool,
    /// Additional details about the successful operation, if any.
    #[serde(rename = "success_detail", skip_serializing_if = "Option::is_none")]
    pub success_detail: Option<String>,
    /// The fully-qualified name (FQN) this of the value that was changed.
    #[serde(rename = "fqn", skip_serializing_if = "Option::is_none")]
    pub fqn: Option<String>,
    /// The environment affected by this step.
    #[serde(rename = "environment")]
    pub environment: Option<String>,
    /// The environment id involved in the operation.
    #[serde(rename = "environment_id", skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    /// The environment name involved in the operation.
    #[serde(rename = "environment_name", skip_serializing_if = "Option::is_none")]
    pub environment_name: Option<String>,
    /// The project affected by this step.
    #[serde(rename = "project")]
    pub project: Option<String>,
    /// The project id involved in the operation.
    #[serde(rename = "project_id", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// The project name involved in the operation.
    #[serde(rename = "project_name", skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// The parameter affected by this step.
    #[serde(rename = "parameter")]
    pub parameter: Option<String>,
    /// The parameter id involved in the operation.
    #[serde(rename = "parameter_id", skip_serializing_if = "Option::is_none")]
    pub parameter_id: Option<String>,
    /// The parameter name involved in the operation.
    #[serde(rename = "parameter_name", skip_serializing_if = "Option::is_none")]
    pub parameter_name: Option<String>,
    /// The integration-native id for the resource.
    #[serde(rename = "venue_id", skip_serializing_if = "Option::is_none")]
    pub venue_id: Option<String>,
    /// The name of the item or resource as known by the integration.
    #[serde(rename = "venue_name", skip_serializing_if = "Option::is_none")]
    pub venue_name: Option<String>,
    /// An error code, if not successful.
    #[serde(rename = "error_code", skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Details on the error that occurred during processing.
    #[serde(rename = "error_detail", skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
}

impl GitHubPullTaskStep {
    /// Pull task step for a GitHub integration.
    pub fn new(
        url: String,
        id: String,
        success: bool,
        environment: Option<String>,
        project: Option<String>,
        parameter: Option<String>,
        created_at: String,
        modified_at: String,
    ) -> GitHubPullTaskStep {
        GitHubPullTaskStep {
            url,
            id,
            operation: None,
            success,
            success_detail: None,
            fqn: None,
            environment,
            environment_id: None,
            environment_name: None,
            project,
            project_id: None,
            project_name: None,
            parameter,
            parameter_id: None,
            parameter_name: None,
            venue_id: None,
            venue_name: None,
            error_code: None,
            error_detail: None,
            created_at,
            modified_at,
        }
    }
}
