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
pub struct Invitation {
    #[serde(rename = "url")]
    pub url: String,
    /// The unique identifier of an invitation.
    #[serde(rename = "id")]
    pub id: String,
    /// The email address of the user to be invited.
    #[serde(rename = "email")]
    pub email: String,
    /// The role that the user will have in the organization, should the user accept.
    #[serde(rename = "role")]
    pub role: Box<crate::models::RoleEnum>,
    /// The user that created the invitation.
    #[serde(rename = "inviter")]
    pub inviter: String,
    /// The current state of the invitation.
    #[serde(rename = "state")]
    pub state: String,
    /// Additional details about the state of the invitation.
    #[serde(rename = "state_detail")]
    pub state_detail: String,
    /// The resulting membership, should the user accept.
    #[serde(rename = "membership")]
    pub membership: String,
}

impl Invitation {
    pub fn new(
        url: String,
        id: String,
        email: String,
        role: crate::models::RoleEnum,
        inviter: String,
        state: String,
        state_detail: String,
        membership: String,
    ) -> Invitation {
        Invitation {
            url,
            id,
            email,
            role: Box::new(role),
            inviter,
            state,
            state_detail,
            membership,
        }
    }
}
