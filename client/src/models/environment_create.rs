/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your parameters and secrets making them easier to manage and use.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentCreate {
    /// The environment name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the environment.  You may find it helpful to document how this environment is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Environments can inherit from a single parent environment which provides values for parameters when specific environments do not have a value set.  Every organization has one default environment that is required to have a value for every parameter in every project.
    #[serde(rename = "parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

impl EnvironmentCreate {
    pub fn new(name: String) -> EnvironmentCreate {
        EnvironmentCreate {
            name,
            description: None,
            parent: None,
        }
    }
}