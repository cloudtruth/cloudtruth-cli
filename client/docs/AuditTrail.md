# AuditTrail

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | A unique identifier for the audit record. | [readonly]
**action** | **String** | The action that was taken. | [readonly]
**object_id** | **String** | The id of the object associated with the action. | [readonly]
**object_name** | **String** | The name of the object associated with the action, if applicable. | [readonly]
**object_type** | [**crate::models::ObjectTypeEnum**](ObjectTypeEnum.md) | The type of object associated with the action. | [readonly]
**timestamp** | **String** | The timestamp of the activity that was audited. | [readonly]
**user** | [**crate::models::User**](User.md) | Details of the user associated with this change. | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


