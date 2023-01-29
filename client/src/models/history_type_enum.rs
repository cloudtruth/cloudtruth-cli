/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum HistoryTypeEnum {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "nothing")]
    Nothing,
    #[serde(rename = "unknown_default_open_api", other)]
    UnknownDefaultOpenApi,
}

impl ToString for HistoryTypeEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Create => String::from("create"),
            Self::Update => String::from("update"),
            Self::Delete => String::from("delete"),
            Self::Nothing => String::from("nothing"),
            Self::UnknownDefaultOpenApi => String::from("unknown_default_open_api"),
        }
    }
}

impl Default for HistoryTypeEnum {
    fn default() -> HistoryTypeEnum {
        Self::Create
    }
}
