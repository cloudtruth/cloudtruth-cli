/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// Value : A value for a parameter in a given environment.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "url")]
    pub url: String,
    /// A unique identifier for the value.
    #[serde(rename = "id")]
    pub id: String,
    /// The environment this value is set in.
    #[serde(rename = "environment")]
    pub environment: String,
    /// The environment name for this value.  This is a convenience to avoid another query against the server to resolve the environment url into a name.
    #[serde(rename = "environment_name")]
    pub environment_name: String,
    /// The earliest tag name this value appears in (within the value's environment).
    #[serde(rename = "earliest_tag")]
    pub earliest_tag: Option<String>,
    /// The parameter this value is for.
    #[serde(rename = "parameter")]
    pub parameter: String,
    /// An external parameter leverages a CloudTruth integration to retrieve content on-demand from an external source.  When this is `false` the value is stored by CloudTruth and considered to be _internal_.  When this is `true`, the `external_fqn` field must be set.
    #[serde(rename = "external", skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,
    /// The FQN, or Fully-Qualified Name, is the path through the integration to get to the desired content.  This must be present and reference a valid integration when the value is `external`.
    #[serde(rename = "external_fqn", skip_serializing_if = "Option::is_none")]
    pub external_fqn: Option<String>,
    /// If the value is `external`, the content returned by the integration can be reduced by applying a JMESpath expression.  This is valid as long as the content is structured and of a supported format.  JMESpath expressions are supported on `json`, `yaml`, and `dotenv` content.
    #[serde(rename = "external_filter", skip_serializing_if = "Option::is_none")]
    pub external_filter: Option<String>,
    /// This field is deprecated and unused.
    #[serde(rename = "external_error")]
    pub external_error: Option<String>,
    /// The most recent mapped pull status for an external value.
    #[serde(rename = "external_status")]
    pub external_status: Option<Box<crate::models::TaskStep>>,
    /// This is the content to use when resolving the Value for an internal non-secret, or when storing a secret.  When storing a secret, this content is stored in your organization's dedicated vault and this field is cleared.  This field is required if the value is being created or updated and is `internal`.  This field cannot be specified when creating or updating an `external` value.
    #[serde(rename = "internal_value", skip_serializing_if = "Option::is_none")]
    pub internal_value: Option<String>,
    /// If `true`, apply template substitution rules to this value.  If `false`, this value is a literal value.  Note: secrets cannot be interpolated.
    #[serde(rename = "interpolated", skip_serializing_if = "Option::is_none")]
    pub interpolated: Option<bool>,
    /// This is the actual content of the Value for the given parameter in the given environment.  If you request secret masking, no secret content will be included in the result and instead a series of asterisks will be used instead for the value.  If you request wrapping, the secret content will be wrapped in an envelope that is bound to your JWT token.  For more information about secret wrapping, see the docs.  Clients applying this value to a shell environment should set `<parameter_name>=<value>` even if `value` is the empty string.  If `value` is `null`, the client should unset that shell environment variable.
    #[serde(rename = "value")]
    pub value: Option<String>,
    /// If true, the `value` field has undergone template evaluation.
    #[serde(rename = "evaluated")]
    pub evaluated: bool,
    /// Indicates the value content is a secret.  Normally this is `true` when the parameter is a secret. It is possible for a parameter to be a secret with a external value that is not a secret.  It is not possible to convert a parameter from a secret to a non-secret if any of the values are external and a secret.  Clients can check this condition by leveraging this field.  It is also possible for a parameter to not be a secret but for this value to be dynamic and reference a Parameter that is a secret.  In this case, we indicate the value is a secret.
    #[serde(rename = "secret")]
    pub secret: Option<bool>,
    /// The parameters this value references, if interpolated.
    #[serde(rename = "referenced_parameters")]
    pub referenced_parameters: Vec<String>,
    /// The templates this value references, if interpolated.
    #[serde(rename = "referenced_templates")]
    pub referenced_templates: Vec<String>,
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
        environment_name: String,
        earliest_tag: Option<String>,
        parameter: String,
        external_error: Option<String>,
        external_status: Option<crate::models::TaskStep>,
        value: Option<String>,
        evaluated: bool,
        secret: Option<bool>,
        referenced_parameters: Vec<String>,
        referenced_templates: Vec<String>,
        created_at: String,
        modified_at: String,
    ) -> Value {
        Value {
            url,
            id,
            environment,
            environment_name,
            earliest_tag,
            parameter,
            external: None,
            external_fqn: None,
            external_filter: None,
            external_error,
            external_status: external_status.map(Box::new),
            internal_value: None,
            interpolated: None,
            value,
            evaluated,
            secret,
            referenced_parameters,
            referenced_templates,
            created_at,
            modified_at,
        }
    }
}
