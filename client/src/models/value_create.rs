/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your parameters and secrets making them easier to manage and use.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// ValueCreate : A value for a parameter in a given environment.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueCreate {
    /// The environment this value is set in.
    #[serde(rename = "environment")]
    pub environment: String,
    /// A dynamic parameter leverages a CloudTruth integration to retrieve content on-demand from an external source.  When this is `false` the value is stored by CloudTruth.  When this is `true`, the `fqn` field must be set.
    #[serde(rename = "dynamic", skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<bool>,
    /// The FQN, or Fully-Qualified Name, is the path through the integration to get to the desired content.  This must be present and reference a valid integration when the value is `dynamic`.
    #[serde(rename = "dynamic_fqn", skip_serializing_if = "Option::is_none")]
    pub dynamic_fqn: Option<String>,
    /// If `dynamic`, the content returned by the integration can be reduced by applying a JMESpath expression.  This is valid as long as the content is structured and of a supported format.  We support JMESpath expressions on `json`, `yaml`, and `dotenv` content.
    #[serde(rename = "dynamic_filter", skip_serializing_if = "Option::is_none")]
    pub dynamic_filter: Option<String>,
    /// This is the content to use when resolving the Value for a static non-secret.
    #[serde(rename = "static_value", skip_serializing_if = "Option::is_none")]
    pub static_value: Option<String>,
}

impl ValueCreate {
    /// A value for a parameter in a given environment.
    pub fn new(environment: String) -> ValueCreate {
        ValueCreate {
            environment,
            dynamic: None,
            dynamic_fqn: None,
            dynamic_filter: None,
            static_value: None,
        }
    }
}
