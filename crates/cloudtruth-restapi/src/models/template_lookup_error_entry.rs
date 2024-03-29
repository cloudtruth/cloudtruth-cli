/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TemplateLookupErrorEntry {
    /// The parameter id.
    #[serde(rename = "parameter_id")]
    pub parameter_id: String,
    /// The parameter name.
    #[serde(rename = "parameter_name")]
    pub parameter_name: String,
    /// The error code.
    #[serde(rename = "error_code")]
    pub error_code: String,
    /// Details about the error.
    #[serde(rename = "error_detail")]
    pub error_detail: String,
}

impl TemplateLookupErrorEntry {
    pub fn new(
        parameter_id: String,
        parameter_name: String,
        error_code: String,
        error_detail: String,
    ) -> TemplateLookupErrorEntry {
        TemplateLookupErrorEntry {
            parameter_id,
            parameter_name,
            error_code,
            error_detail,
        }
    }
}
