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
pub struct DiscoveryResult {
    #[serde(rename = "matched")]
    pub matched: ::std::collections::HashMap<String, crate::models::DiscoveredContent>,
    #[serde(rename = "skipped")]
    pub skipped: ::std::collections::HashMap<String, String>,
}

impl DiscoveryResult {
    pub fn new(
        matched: ::std::collections::HashMap<String, crate::models::DiscoveredContent>,
        skipped: ::std::collections::HashMap<String, String>,
    ) -> DiscoveryResult {
        DiscoveryResult { matched, skipped }
    }
}
