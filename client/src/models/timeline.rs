/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    /// If present, additional history can be retrieved using this timestamp in the next call in the as_of query parameter.
    #[serde(rename = "next_as_of", skip_serializing_if = "Option::is_none")]
    pub next_as_of: Option<String>,
    #[serde(rename = "results")]
    pub results: Vec<crate::models::TimelineEntry>,
}

impl Timeline {
    pub fn new(results: Vec<crate::models::TimelineEntry>) -> Timeline {
        Timeline {
            next_as_of: None,
            results,
        }
    }
}
