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
pub struct OrganizationCreate {
    /// The organization name.
    #[serde(rename = "name")]
    pub name: String,
}

impl OrganizationCreate {
    pub fn new(name: String) -> OrganizationCreate {
        OrganizationCreate { name }
    }
}
