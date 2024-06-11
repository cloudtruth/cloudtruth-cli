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
pub enum ParameterRuleTypeEnum {
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "min_len")]
    MinLen,
    #[serde(rename = "max_len")]
    MaxLen,
    #[serde(rename = "regex")]
    Regex,
    #[serde(rename = "unknown_default_open_api", other)]
    UnknownDefaultOpenApi,
}

impl ToString for ParameterRuleTypeEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Min => String::from("min"),
            Self::Max => String::from("max"),
            Self::MinLen => String::from("min_len"),
            Self::MaxLen => String::from("max_len"),
            Self::Regex => String::from("regex"),
            Self::UnknownDefaultOpenApi => String::from("unknown_default_open_api"),
        }
    }
}

impl Default for ParameterRuleTypeEnum {
    fn default() -> ParameterRuleTypeEnum {
        Self::Min
    }
}
