/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your parameters and secrets making them easier to manage and use.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// Value : A value for a parameter in a given environment.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "url")]
    pub url: String,
    /// A unique identifier for the value.
    #[serde(rename = "id")]
    pub id: String,
    /// The environment this value is set in.
    #[serde(rename = "environment")]
    pub environment: String,
    /// The parameter this value is for.
    #[serde(rename = "parameter")]
    pub parameter: String,
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
    /// This is the actual content of the Value for the given parameter in the given environment.  Depending on the settings in the Value, the following things occur to calculate the `value`:  For values that are not `dynamic` and parameters that are not `secret`, the system will use the content in `static_value` to satisfy the request.  For values that are not `dynamic` and parameters that are `secret`, the system will retrieve the content from your organization's dedicated vault.  For values that are `dynamic`, the system will retrieve the content from the integration on-demand.  If the content from the integration is `secret` and the parameter is not, or if the parameter is `secret` and the content from the integration is not, an error response will be given.  If a `dynamic_filter` is present then the content will have a JMESpath query applied, and that becomes the resulting value.  If you request secret masking, no secret content will be included in the result and instead a series of asterisks will be used instead for the value.  If you request wrapping, the secret content will be wrapped in an envelope that is bound to your JWT token.  For more information about secret wrapping, see the docs.  Clients applying this value to a shell environment should set `<parameter_name>=<value>` even if `value` is the empty string.  If `value` is `null`, the client should unset that shell environment variable.
    #[serde(rename = "value")]
    pub value: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "modified_at")]
    pub modified_at: String,
}

impl Value {
    /// A value for a parameter in a given environment.
    pub fn new(
        url: String,
        id: String,
        environment: String,
        parameter: String,
        value: Option<String>,
        created_at: String,
        modified_at: String,
    ) -> Value {
        Value {
            url,
            id,
            environment,
            parameter,
            dynamic: None,
            dynamic_fqn: None,
            dynamic_filter: None,
            static_value: None,
            value,
            created_at,
            modified_at,
        }
    }
}
