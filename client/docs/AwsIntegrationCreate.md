# AwsIntegrationCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | Option<**String**> | The optional description for the integration. | [optional]
**aws_account_id** | **String** | The AWS Account ID. | 
**aws_enabled_regions** | [**Vec<crate::models::AwsEnabledRegionsEnum>**](AwsEnabledRegionsEnum.md) | The AWS regions to integrate with. | 
**aws_enabled_services** | [**Vec<crate::models::AwsEnabledServicesEnum>**](AwsEnabledServicesEnum.md) | The AWS services to integrate with. | 
**aws_external_id** | Option<**String**> | This is a shared secret between the AWS Administrator who set up your IAM trust relationship and your CloudTruth AWS Integration.  If your AWS Administrator provided you with a value use it, otherwise we will generate a random value for you to give to your AWS Administrator. | [optional]
**aws_role_name** | **String** | The role that CloudTruth will assume when interacting with your AWS Account through this integration.  The role is configured by your AWS Account Administrator.  If your AWS Administrator provided you with a value use it, otherwise make your own role name and give it to your AWS Administrator. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


