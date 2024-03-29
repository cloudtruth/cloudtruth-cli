/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// PatchedTemplateUpdate : A parameter template in a given project, optionally instantiated against an environment.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PatchedTemplateUpdate {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The template name.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// ('A description of the template.  You may find it helpful to document how this template is used to assist others when they need to maintain software that uses this content.',)
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// If true, the `body` field has undergone evaluation.
    #[serde(rename = "evaluated", skip_serializing_if = "Option::is_none")]
    pub evaluated: Option<bool>,
    /// The content of the template.  Use mustache-style templating delimiters of `{{` and `}}` to reference parameter values by name for substitution into the template result.
    #[serde(rename = "body", skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl PatchedTemplateUpdate {
    /// A parameter template in a given project, optionally instantiated against an environment.
    pub fn new() -> PatchedTemplateUpdate {
        PatchedTemplateUpdate {
            id: None,
            name: None,
            description: None,
            evaluated: None,
            body: None,
            created_at: None,
            modified_at: None,
        }
    }
}
