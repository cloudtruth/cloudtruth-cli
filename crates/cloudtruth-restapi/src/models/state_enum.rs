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
pub enum StateEnum {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "unknown_default_open_api", other)]
    UnknownDefaultOpenApi,
}

impl ToString for StateEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Queued => String::from("queued"),
            Self::Running => String::from("running"),
            Self::Skipped => String::from("skipped"),
            Self::Success => String::from("success"),
            Self::Failure => String::from("failure"),
            Self::UnknownDefaultOpenApi => String::from("unknown_default_open_api"),
        }
    }
}

impl Default for StateEnum {
    fn default() -> StateEnum {
        Self::Queued
    }
}
