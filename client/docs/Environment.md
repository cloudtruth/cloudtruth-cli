# Environment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | A unique identifier for the environment. | [readonly]
**name** | **String** | The environment name. | 
**description** | Option<**String**> | A description of the environment.  You may find it helpful to document how this environment is used to assist others when they need to maintain software that uses this content. | [optional]
**parent** | Option<**String**> | Environments can inherit from a single parent environment which provides values for parameters when specific environments do not have a value set.  Every organization has one default environment that is required to have a value for every parameter in every project. | [optional]
**created_at** | **String** |  | [readonly]
**modified_at** | **String** |  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


