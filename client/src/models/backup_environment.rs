/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// BackupEnvironment : Basic environment data at a point in time.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BackupEnvironment {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl BackupEnvironment {
    /// Basic environment data at a point in time.
    pub fn new(name: String) -> BackupEnvironment {
        BackupEnvironment {
            name,
            parent: None,
            description: None,
        }
    }
}


