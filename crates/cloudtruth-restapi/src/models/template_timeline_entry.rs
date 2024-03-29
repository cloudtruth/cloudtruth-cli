/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// TemplateTimelineEntry : Details about a single change.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TemplateTimelineEntry {
    #[serde(rename = "history_type")]
    pub history_type: Option<Box<crate::models::HistoryTypeEnum>>,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
    #[serde(rename = "modified_by", skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
    /// The template record as it was when archived for history.
    #[serde(rename = "history_template")]
    pub history_template: Option<Box<crate::models::TemplateTimelineEntryTemplate>>,
}

impl TemplateTimelineEntry {
    /// Details about a single change.
    pub fn new(
        history_type: Option<crate::models::HistoryTypeEnum>,
        modified_at: Option<String>,
        history_template: Option<crate::models::TemplateTimelineEntryTemplate>,
    ) -> TemplateTimelineEntry {
        TemplateTimelineEntry {
            history_type: history_type.map(Box::new),
            modified_at,
            modified_by: None,
            history_template: history_template.map(Box::new),
        }
    }
}
