/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// TemplateLookupError : Indicates errors occurred while retrieving values to substitute into the template.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TemplateLookupError {
    #[serde(rename = "detail")]
    pub detail: Vec<crate::models::TemplateLookupErrorEntry>,
}

impl TemplateLookupError {
    /// Indicates errors occurred while retrieving values to substitute into the template.
    pub fn new(detail: Vec<crate::models::TemplateLookupErrorEntry>) -> TemplateLookupError {
        TemplateLookupError { detail }
    }
}