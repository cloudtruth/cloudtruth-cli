/*
 * CloudTruth Management API
 *
 * CloudTruth centralizes your configuration parameters and secrets making them easier to manage and use as a team.
 *
 * The version of the OpenAPI document: v1
 * Contact: support@cloudtruth.com
 * Generated by: https://openapi-generator.tech
 */

/// PatchedValueUpdate : Unlike other UpdateSerializers, we do not inherit from the CreateSerializer here because `environment` is not a required field for updates.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PatchedValueUpdate {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// An external parameter leverages a CloudTruth integration to retrieve content on-demand from an external source.  When this is `false` the value is stored by CloudTruth and considered to be _internal_.  When this is `true`, the `external_fqn` field must be set.
    #[serde(rename = "external", skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,
    /// The FQN, or Fully-Qualified Name, is the path through the integration to get to the desired content.  This must be present and reference a valid integration when the value is `external`.
    #[serde(rename = "external_fqn", skip_serializing_if = "Option::is_none")]
    pub external_fqn: Option<String>,
    /// If the value is `external`, the content returned by the integration can be reduced by applying a JMESpath expression.  This is valid as long as the content is structured and of a supported format.  JMESpath expressions are supported on `json`, `yaml`, and `dotenv` content.
    #[serde(rename = "external_filter", skip_serializing_if = "Option::is_none")]
    pub external_filter: Option<String>,
    /// This is the content to use when resolving the Value for an internal non-secret, or when storing a secret.  This field cannot be specified when creating or updating an `external` value.
    #[serde(rename = "internal_value", skip_serializing_if = "Option::is_none")]
    pub internal_value: Option<String>,
    /// If `true`, apply template substitution rules to this value.  If `false`, this value is a literal value.  Note: secrets cannot be interpolated.
    #[serde(rename = "interpolated", skip_serializing_if = "Option::is_none")]
    pub interpolated: Option<bool>,
    /// Indicates the value content is a secret.  Normally this is `true` when the parameter is a secret. It is possible for a parameter to be a secret with a external value that is not a secret.  It is not possible to convert a parameter from a secret to a non-secret if any of the values are external and a secret.  Clients can check this condition by leveraging this field.  It is also possible for a parameter to not be a secret but for this value to be dynamic and reference a Parameter that is a secret.  In this case, we indicate the value is a secret.
    #[serde(rename = "secret", skip_serializing_if = "Option::is_none")]
    pub secret: Option<bool>,
    /// This is the actual content of the Value for the given parameter in the given environment.  If you request secret masking, no secret content will be included in the result and instead a series of asterisks will be used instead for the value.  Clients applying this value to a shell environment should set `<parameter_name>=<value>` even if `value` is the empty string.  If `value` is `null`, the client should unset that shell environment variable.
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "modified_at", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl PatchedValueUpdate {
    /// Unlike other UpdateSerializers, we do not inherit from the CreateSerializer here because `environment` is not a required field for updates.
    pub fn new() -> PatchedValueUpdate {
        PatchedValueUpdate {
            id: None,
            external: None,
            external_fqn: None,
            external_filter: None,
            internal_value: None,
            interpolated: None,
            secret: None,
            value: None,
            created_at: None,
            modified_at: None,
        }
    }
}