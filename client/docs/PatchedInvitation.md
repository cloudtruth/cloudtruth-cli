# PatchedInvitation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | Option<**String**> |  | [optional][readonly]
**id** | Option<**String**> | The unique identifier of an invitation. | [optional][readonly]
**email** | Option<**String**> | The email address of the user to be invited. | [optional]
**role** | Option<[**crate::models::RoleEnum**](RoleEnum.md)> | The role that the user will have in the organization, should the user accept. | [optional]
**inviter** | Option<**String**> | The user that created the invitation. | [optional][readonly]
**state** | Option<**String**> | The current state of the invitation. | [optional][readonly]
**state_detail** | Option<**String**> | Additional details about the state of the invitation. | [optional][readonly]
**membership** | Option<**String**> | The resulting membership, should the user accept. | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


