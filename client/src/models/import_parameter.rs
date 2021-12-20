/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// ImportParameter : Describes an imported parameter.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportParameter {
    /// Project name
    #[serde(rename = "project_name")]
    pub project_name: String,
    /// Project identifier
    #[serde(rename = "project_id", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Environment where parameter is defined
    #[serde(rename = "environment_name")]
    pub environment_name: String,
    /// Environment identifier where value is set
    #[serde(rename = "environment_id", skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    /// Parameter name
    #[serde(rename = "parameter_name")]
    pub parameter_name: String,
    /// Parameter identifier
    #[serde(rename = "parameter_id", skip_serializing_if = "Option::is_none")]
    pub parameter_id: Option<String>,
    /// Whether to treat the parameter as a secret
    #[serde(rename = "secret", skip_serializing_if = "Option::is_none")]
    pub secret: Option<bool>,
    /// Parameter value in the specified environment
    #[serde(rename = "value")]
    pub value: String,
    /// Parameter value identifier in the environment
    #[serde(rename = "value_id", skip_serializing_if = "Option::is_none")]
    pub value_id: Option<String>,
    /// Date and time the parameter value was created
    #[serde(rename = "created_at")]
    pub created_at: Option<String>,
    /// Date and time the parameter value was updated
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
    /// Action taken on environment value for parameter
    #[serde(rename = "action")]
    pub action: String,
}

impl ImportParameter {
    /// Describes an imported parameter.
    pub fn new(
        project_name: String,
        environment_name: String,
        parameter_name: String,
        value: String,
        created_at: Option<String>,
        modified_at: Option<String>,
        action: String,
    ) -> ImportParameter {
        ImportParameter {
            project_name,
            project_id: None,
            environment_name,
            environment_id: None,
            parameter_name,
            parameter_id: None,
            secret: None,
            value,
            value_id: None,
            created_at,
            modified_at,
            action,
        }
    }
}
