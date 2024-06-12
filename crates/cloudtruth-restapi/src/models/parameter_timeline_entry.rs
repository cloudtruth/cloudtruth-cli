/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// ParameterTimelineEntry : Details about a single change.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ParameterTimelineEntry {
    #[serde(rename = "history_type")]
    pub history_type: Option<Box<crate::models::HistoryTypeEnum>>,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
    #[serde(rename = "modified_by", skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
    /// The affected environment(s).
    #[serde(rename = "history_environments")]
    pub history_environments: Vec<crate::models::ParameterTimelineEntryEnvironment>,
    /// The component of the parameter that changed.
    #[serde(rename = "history_model")]
    pub history_model: Option<Box<crate::models::HistoryModelEnum>>,
    /// The affected parameter.
    #[serde(rename = "history_parameter")]
    pub history_parameter: Option<Box<crate::models::ParameterTimelineEntryParameter>>,
}

impl ParameterTimelineEntry {
    /// Details about a single change.
    pub fn new(
        history_type: Option<crate::models::HistoryTypeEnum>,
        modified_at: Option<String>,
        history_environments: Vec<crate::models::ParameterTimelineEntryEnvironment>,
        history_model: Option<crate::models::HistoryModelEnum>,
        history_parameter: Option<crate::models::ParameterTimelineEntryParameter>,
    ) -> ParameterTimelineEntry {
        ParameterTimelineEntry {
            history_type: history_type.map(Box::new),
            modified_at,
            modified_by: None,
            history_environments,
            history_model: history_model.map(Box::new),
            history_parameter: history_parameter.map(Box::new),
        }
    }
}
