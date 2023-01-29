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
pub enum NodeTypeEnum {
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "value")]
    Value,
    #[serde(rename = "unknown_default_open_api")]
    UnknownDefaultOpenApi,

}

impl ToString for NodeTypeEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Directory => String::from("directory"),
            Self::File => String::from("file"),
            Self::Value => String::from("value"),
            Self::UnknownDefaultOpenApi => String::from("unknown_default_open_api"),
        }
    }
}

impl Default for NodeTypeEnum {
    fn default() -> NodeTypeEnum {
        Self::Directory
    }
}




