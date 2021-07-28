/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// Parameter : A single parameter inside of a project.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(rename = "url")]
    pub url: String,
    /// A unique identifier for the parameter.
    #[serde(rename = "id")]
    pub id: String,
    /// The parameter name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the parameter.  You may find it helpful to document how this parameter is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Indicates if this content is secret or not.  When a parameter is considered to be a secret, any static values are stored in a dedicated vault for your organization for maximum security.  Dynamic values are inspected on-demand to ensure they align with the parameter's secret setting and if they do not, those dynamic values are not allowed to be used.
    #[serde(rename = "secret", skip_serializing_if = "Option::is_none")]
    pub secret: Option<bool>,
    #[serde(rename = "templates")]
    pub templates: Vec<String>,
    ///              Each parameter has an effective value in every environment based on             environment inheritance and which environments have had a value set.              Environments inherit from a single parent to form a tree, as a result             a single parameter may have different values present for each environment.             When a value is not explicitly set in an environment, the parent environment             is consulted to see if it has a value defined, and so on.              The dictionary of values has an environment url as the key, and the optional             Value record that it resolves to.  If the Value.environment matches the key,             then it is an explicit value set for that environment.  If they differ, the             value was obtained from a parent environment (directly or indirectly).  If the             value is None then no value has ever been set in any environment for this             parameter.              key: Environment url             value: optional Value record         
    #[serde(rename = "values")]
    pub values: ::std::collections::HashMap<String, Option<crate::models::Value>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
}

impl Parameter {
    /// A single parameter inside of a project.
    pub fn new(
        url: String,
        id: String,
        name: String,
        templates: Vec<String>,
        values: ::std::collections::HashMap<String, Option<crate::models::Value>>,
        created_at: String,
        modified_at: String,
    ) -> Parameter {
        Parameter {
            url,
            id,
            name,
            description: None,
            secret: None,
            templates,
            values,
            created_at,
            modified_at,
        }
    }
}
