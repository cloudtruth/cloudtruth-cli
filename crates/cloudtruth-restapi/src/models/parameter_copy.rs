/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// ParameterCopy : A single parameter inside of a project.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ParameterCopy {
    /// The parameter name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the parameter.  You may find it helpful to document how this parameter is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The project url.
    #[serde(rename = "project")]
    pub project: String,
}

impl ParameterCopy {
    /// A single parameter inside of a project.
    pub fn new(name: String, project: String) -> ParameterCopy {
        ParameterCopy {
            name,
            description: None,
            project,
        }
    }
}
