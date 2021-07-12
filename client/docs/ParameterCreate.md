# ParameterCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The parameter name. | 
**description** | Option<**String**> | A description of the parameter.  You may find it helpful to document how this parameter is used to assist others when they need to maintain software that uses this content. | [optional]
**secret** | Option<**bool**> | Indicates if this content is secret or not.  When a parameter is considered to be a secret, any static values are stored in a dedicated vault for your organization for maximum security.  Dynamic values are inspected on-demand to ensure they align with the parameter's secret setting and if they do not, those dynamic values are not allowed to be used. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


