/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct EnvironmentUpdate {
    #[serde(rename = "id")]
    pub id: String,
    /// The environment name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the environment.  You may find it helpful to document how this environment is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Environments can inherit from a single parent environment which provides values for parameters when specific environments do not have a value set.  Every organization has one default environment that cannot be removed.
    #[serde(rename = "parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// This is the opposite of `parent`, see that field for more details.
    #[serde(rename = "children")]
    pub children: Vec<String>,
    /// Indicates if access control is being enforced through grants.
    #[serde(rename = "access_controlled", skip_serializing_if = "Option::is_none")]
    pub access_controlled: Option<bool>,
    /// Your role in the environment, if the environment is access-controlled.
    #[serde(rename = "role")]
    pub role: Option<Box<crate::models::RoleEnum>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: Option<String>,
}

impl EnvironmentUpdate {
    pub fn new(
        id: String,
        name: String,
        children: Vec<String>,
        role: Option<crate::models::RoleEnum>,
        created_at: String,
        modified_at: Option<String>,
    ) -> EnvironmentUpdate {
        EnvironmentUpdate {
            id,
            name,
            description: None,
            parent: None,
            children,
            access_controlled: None,
            role: role.map(Box::new),
            created_at,
            modified_at,
        }
    }
}
