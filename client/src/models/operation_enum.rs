/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum OperationEnum {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
}

impl ToString for OperationEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Create => String::from("create"),
            Self::Read => String::from("read"),
            Self::Update => String::from("update"),
            Self::Delete => String::from("delete"),
        }
    }
}

impl Default for OperationEnum {
    fn default() -> OperationEnum {
        Self::Create
    }
}
