# PatchedAwsIntegration

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | Option<**String**> |  | [optional][readonly]
**id** | Option<**String**> | The unique identifier for the integration. | [optional][readonly]
**name** | Option<**String**> |  | [optional][readonly]
**description** | Option<**String**> | The optional description for the integration. | [optional]
**status** | Option<**String**> | The status of the integration connection with the third-party provider as of the `status_last_checked_at` field.  The status is updated automatically by the server when the integration is modified. | [optional][readonly]
**status_detail** | Option<**String**> | If an error occurs, more details will be available in this field. | [optional][readonly]
**status_last_checked_at** | Option<**String**> | The last time the status was evaluated. | [optional][readonly]
**_type** | Option<**String**> | The type of integration. | [optional][readonly]
**created_at** | Option<**String**> |  | [optional][readonly]
**modified_at** | Option<**String**> |  | [optional][readonly]
**fqn** | Option<**String**> |  | [optional][readonly]
**aws_account_id** | Option<**String**> | The AWS Account ID. | [optional]
**aws_enabled_regions** | Option<[**Vec<crate::models::AwsEnabledRegionsEnum>**](AwsEnabledRegionsEnum.md)> | The AWS regions to integrate with. | [optional]
**aws_enabled_services** | Option<[**Vec<crate::models::AwsEnabledServicesEnum>**](AwsEnabledServicesEnum.md)> | The AWS services to integrate with. | [optional]
**aws_external_id** | Option<**String**> | This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator. | [optional]
**aws_role_name** | Option<**String**> | The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


