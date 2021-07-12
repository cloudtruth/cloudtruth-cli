# Invitation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | The unique identifier of an invitation. | [readonly]
**email** | **String** | The email address of the user to be invited. | 
**role** | [**crate::models::RoleEnum**](RoleEnum.md) | The role that the user will have in the organization, should the user accept. | 
**inviter** | **String** | The user that created the invitation. | [readonly]
**state** | **String** | The current state of the invitation. | [readonly]
**state_detail** | **String** | Additional details about the state of the invitation. | [readonly]
**membership** | **String** | The resulting membership, should the user accept. | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


