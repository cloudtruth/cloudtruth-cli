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
pub struct ProjectCreate {
    /// The project name.
    #[serde(rename = "name")]
    pub name: String,
    /// A description of the project.  You may find it helpful to document how this project is used to assist others when they need to maintain software that uses this content.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A regular expression parameter names must match
    #[serde(
        rename = "parameter_name_pattern",
        skip_serializing_if = "Option::is_none"
    )]
    pub parameter_name_pattern: Option<String>,
    /// Project dependencies allow projects to be used for shared configuration, for example a database used by many applications needs to advertise its port number.  Projects can depend on another project which will add the parameters from the parent project into the current project.  All of the parameter names between the two projects must be unique.  When retrieving values or rendering templates, all of the parameters from the parent project will also be available in the current project.
    #[serde(rename = "depends_on", skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<String>,
}

impl ProjectCreate {
    pub fn new(name: String) -> ProjectCreate {
        ProjectCreate {
            name,
            description: None,
            parameter_name_pattern: None,
            depends_on: None,
        }
    }
}
