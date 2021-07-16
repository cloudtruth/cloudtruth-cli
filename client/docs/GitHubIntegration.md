# GitHubIntegration

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | The unique identifier for the integration. | [readonly]
**name** | **String** |  | [readonly]
**description** | Option<**String**> | The optional description for the integration. | [optional]
**status** | **String** | The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified. | [readonly]
**status_detail** | **String** | If an error occurs, more details will be available in this field. | [readonly]
**status_last_checked_at** | **String** | The last time the status was evaluated. | [readonly]
**_type** | **String** | The type of integration. | [readonly]
**created_at** | **String** |  | [readonly]
**modified_at** | **String** |  | [readonly]
**fqn** | **String** |  | [readonly]
**gh_installation_id** | **i32** |  | 
**gh_organization_slug** | **String** |  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


