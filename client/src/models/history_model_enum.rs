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
pub enum HistoryModelEnum {
    #[serde(rename = "Parameter")]
    Parameter,
    #[serde(rename = "ParameterRule")]
    ParameterRule,
    #[serde(rename = "Value")]
    Value,
    #[serde(rename = "unknown_default_open_api")]
    UnknownDefaultOpenApi,

}

impl ToString for HistoryModelEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Parameter => String::from("Parameter"),
            Self::ParameterRule => String::from("ParameterRule"),
            Self::Value => String::from("Value"),
            Self::UnknownDefaultOpenApi => String::from("unknown_default_open_api"),
        }
    }
}

impl Default for HistoryModelEnum {
    fn default() -> HistoryModelEnum {
        Self::Parameter
    }
}




