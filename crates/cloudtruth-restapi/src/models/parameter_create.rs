/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// ParameterCreate : A single parameter inside of a project.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ParameterCreate {
    /// The parameter name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the parameter.  You may find it helpful to document how this parameter is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Indicates if this content is secret or not.  External values are inspected on-demand to ensure they align with the parameter's secret setting and if they do not, those external values are not allowed to be used.
    #[serde(rename = "secret", skip_serializing_if = "Option::is_none")]
    pub secret: Option<bool>,
    /// The type of this Parameter.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
}

impl ParameterCreate {
    /// A single parameter inside of a project.
    pub fn new(name: String) -> ParameterCreate {
        ParameterCreate {
            name,
            description: None,
            secret: None,
            _type: None,
        }
    }
}
