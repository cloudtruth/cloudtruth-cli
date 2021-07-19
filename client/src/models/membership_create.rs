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
pub struct MembershipCreate {
    /// The user of the membership.
    #[serde(rename = "user")]
    pub user: String,
    /// The role that the user has in the organization.
    #[serde(rename = "role")]
    pub role: Box<crate::models::RoleEnum>,
}

impl MembershipCreate {
    pub fn new(user: String, role: crate::models::RoleEnum) -> MembershipCreate {
        MembershipCreate {
            user,
            role: Box::new(role),
        }
    }
}