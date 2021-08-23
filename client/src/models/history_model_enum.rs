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
pub enum HistoryModelEnum {
    #[serde(rename = "Parameter")]
    Parameter,
    #[serde(rename = "ParameterRule")]
    ParameterRule,
    #[serde(rename = "Value")]
    Value,
}

impl ToString for HistoryModelEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Parameter => String::from("Parameter"),
            Self::ParameterRule => String::from("ParameterRule"),
            Self::Value => String::from("Value"),
        }
    }
}
