/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your parameters and secrets making them easier to manage and use.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RoleEnum {
    #[serde(rename = "OWNER")]
    OWNER,
    #[serde(rename = "ADMIN")]
    ADMIN,
    #[serde(rename = "CONTRIB")]
    CONTRIB,
    #[serde(rename = "VIEWER")]
    VIEWER,
}

impl ToString for RoleEnum {
    fn to_string(&self) -> String {
        match self {
            Self::OWNER => String::from("OWNER"),
            Self::ADMIN => String::from("ADMIN"),
            Self::CONTRIB => String::from("CONTRIB"),
            Self::VIEWER => String::from("VIEWER"),
        }
    }
}
