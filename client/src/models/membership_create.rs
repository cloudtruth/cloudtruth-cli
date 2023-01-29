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
pub struct MembershipCreate {
    /// The user of the membership.
    #[serde(rename = "user")]
    pub user: String,
    /// The role that the user has in the organization.
    #[serde(rename = "role")]
    pub role: Option<Box<crate::models::RoleEnum>>,
}

impl MembershipCreate {
    pub fn new(user: String, role: Option<crate::models::RoleEnum>) -> MembershipCreate {
        MembershipCreate {
            user,
            role: Box::new(role),
        }
    }
}


