/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct DiscoveredContent {
    #[serde(rename = "venue_id")]
    pub venue_id: String,
    #[serde(rename = "venue_name")]
    pub venue_name: String,
    #[serde(rename = "environment_name")]
    pub environment_name: String,
    #[serde(rename = "project_name")]
    pub project_name: String,
    #[serde(rename = "parameter_name")]
    pub parameter_name: String,
}

impl DiscoveredContent {
    pub fn new(
        venue_id: String,
        venue_name: String,
        environment_name: String,
        project_name: String,
        parameter_name: String,
    ) -> DiscoveredContent {
        DiscoveredContent {
            venue_id,
            venue_name,
            environment_name,
            project_name,
            parameter_name,
        }
    }
}
