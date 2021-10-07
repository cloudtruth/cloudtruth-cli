/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// AwsPushUpdate : Update a push.  The `region` and `service` cannot be changed on an existing push.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AwsPushUpdate {
    #[serde(rename = "url")]
    pub url: String,
    /// The unique identifier for the push action.
    #[serde(rename = "id")]
    pub id: String,
    /// The push action name.
    #[serde(rename = "name")]
    pub name: String,
    /// The optional description for the push action.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Projects that are included in the push.
    #[serde(rename = "projects")]
    pub projects: Vec<String>,
    /// Tags are used to select parameters by environment from the projects included in the push.  You cannot have two tags from the same environment in the same push.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
    /// Defines a path through the integration to the location where values will be pushed.  The following mustache-style substitutions can be used in the string:    - ``{{ environment }}`` to insert the environment name   - ``{{ parameter }}`` to insert the parameter name   - ``{{ project }}`` to insert the project name   - ``{{ push }}`` to insert the push name   - ``{{ tag }}`` to insert the tag name  We recommend that you use project, environment, and parameter at a minimum to disambiguate your pushed resource identifiers.
    #[serde(rename = "resource")]
    pub resource: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
}

impl AwsPushUpdate {
    /// Update a push.  The `region` and `service` cannot be changed on an existing push.
    pub fn new(
        url: String,
        id: String,
        name: String,
        projects: Vec<String>,
        tags: Vec<String>,
        resource: String,
        created_at: String,
        modified_at: String,
    ) -> AwsPushUpdate {
        AwsPushUpdate {
            url,
            id,
            name,
            description: None,
            projects,
            tags,
            resource,
            created_at,
            modified_at,
        }
    }
}
