# Template

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | A unique identifier for the template. | [readonly]
**name** | **String** | The template name. | 
**description** | Option<**String**> | A description of the template.  You may find it helpful to document how this template is used to assist others when they need to maintain software that uses this content. | [optional]
**body** | Option<**String**> | The content of the template.  Use mustache-style templating delimiters of `{{` and `}}` to reference parameter values by name for substitution into the template result. | [optional]
**parameters** | **Vec<String>** |  | [readonly]
**created_at** | **String** |  | [readonly]
**modified_at** | **String** |  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


