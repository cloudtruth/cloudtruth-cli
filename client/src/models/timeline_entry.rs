/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// TimelineEntry : Details about a single change.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimelineEntry {
    #[serde(rename = "history_date")]
    pub history_date: String,
    /// The affected environment(s).
    #[serde(rename = "history_environments")]
    pub history_environments: Vec<crate::models::TimelineEntryEnvironment>,
    /// The component of the parameter that changed.
    #[serde(rename = "history_model")]
    pub history_model: Box<crate::models::HistoryModelEnum>,
    /// The affected parameter.
    #[serde(rename = "history_parameter")]
    pub history_parameter: Box<crate::models::TimelineEntryParameter>,
    #[serde(rename = "history_type")]
    pub history_type: Box<crate::models::HistoryTypeEnum>,
    /// The unique identifier of a user.
    #[serde(rename = "history_user", skip_serializing_if = "Option::is_none")]
    pub history_user: Option<String>,
}

impl TimelineEntry {
    /// Details about a single change.
    pub fn new(
        history_date: String,
        history_environments: Vec<crate::models::TimelineEntryEnvironment>,
        history_model: crate::models::HistoryModelEnum,
        history_parameter: crate::models::TimelineEntryParameter,
        history_type: crate::models::HistoryTypeEnum,
    ) -> TimelineEntry {
        TimelineEntry {
            history_date,
            history_environments,
            history_model: Box::new(history_model),
            history_parameter: Box::new(history_parameter),
            history_type: Box::new(history_type),
            history_user: None,
        }
    }
}
