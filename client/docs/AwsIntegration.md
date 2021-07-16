# AwsIntegration

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
**aws_account_id** | **String** | The AWS Account ID. | 
**aws_enabled_regions** | [**Vec<crate::models::AwsEnabledRegionsEnum>**](AwsEnabledRegionsEnum.md) | The AWS regions to integrate with. | 
**aws_enabled_services** | [**Vec<crate::models::AwsEnabledServicesEnum>**](AwsEnabledServicesEnum.md) | The AWS services to integrate with. | 
**aws_external_id** | Option<**String**> | This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator. | [optional]
**aws_role_name** | **String** | The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


