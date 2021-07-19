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
pub struct PatchedProject {
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A unique identifier for the project.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The project name.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A description of the project.  You may find it helpful to document how this project is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl PatchedProject {
    pub fn new() -> PatchedProject {
        PatchedProject {
            url: None,
            id: None,
            name: None,
            description: None,
            created_at: None,
            modified_at: None,
        }
    }
}