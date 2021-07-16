# ValueCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**environment** | **String** | The environment this value is set in. | 
**dynamic** | Option<**bool**> | A dynamic parameter leverages a CloudTruth integration to retrieve content on-demand from an external source.  When this is `false` the value is stored by CloudTruth.  When this is `true`, the `fqn` field must be set. | [optional]
**dynamic_fqn** | Option<**String**> | The FQN, or Fully-Qualified Name, is the path through the integration to get to the desired content.  This must be present and reference a valid integration when the value is `dynamic`. | [optional]
**dynamic_filter** | Option<**String**> | If `dynamic`, the content returned by the integration can be reduced by applying a JMESpath expression.  This is valid as long as the content is structured and of a supported format.  We support JMESpath expressions on `json`, `yaml`, and `dotenv` content. | [optional]
**static_value** | Option<**String**> | This is the content to use when resolving the Value for a static non-secret. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


