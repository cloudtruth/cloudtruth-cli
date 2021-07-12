# \IntegrationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**integrations_aws_create**](IntegrationsApi.md#integrations_aws_create) | **post** /api/v1/integrations/aws/ | Establishes an AWS Integration.
[**integrations_aws_destroy**](IntegrationsApi.md#integrations_aws_destroy) | **delete** /api/v1/integrations/aws/{id}/ | 
[**integrations_aws_list**](IntegrationsApi.md#integrations_aws_list) | **get** /api/v1/integrations/aws/ | 
[**integrations_aws_partial_update**](IntegrationsApi.md#integrations_aws_partial_update) | **patch** /api/v1/integrations/aws/{id}/ | 
[**integrations_aws_retrieve**](IntegrationsApi.md#integrations_aws_retrieve) | **get** /api/v1/integrations/aws/{id}/ | Get details of an AWS Integration.
[**integrations_aws_update**](IntegrationsApi.md#integrations_aws_update) | **put** /api/v1/integrations/aws/{id}/ | 
[**integrations_explore_list**](IntegrationsApi.md#integrations_explore_list) | **get** /api/v1/integrations/explore/ | Retrieve third-party integration data for the specified FQN.
[**integrations_github_create**](IntegrationsApi.md#integrations_github_create) | **post** /api/v1/integrations/github/ | Establishes a GitHub Integration.
[**integrations_github_destroy**](IntegrationsApi.md#integrations_github_destroy) | **delete** /api/v1/integrations/github/{id}/ | 
[**integrations_github_list**](IntegrationsApi.md#integrations_github_list) | **get** /api/v1/integrations/github/ | 
[**integrations_github_retrieve**](IntegrationsApi.md#integrations_github_retrieve) | **get** /api/v1/integrations/github/{id}/ | Get details of a GitHub Integration.



## integrations_aws_create

> crate::models::AwsIntegration integrations_aws_create(aws_integration_create)
Establishes an AWS Integration.

### Description ###  Establishes an AWS Integration for your CloudTruth organization.  ### Pre-Conditions ###  - An AWS Integration for the account and role cannot already exist. ### Post-Conditions ###  - You must establish an IAM role and trust relationship based on the Role Name and the External ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aws_integration_create** | [**AwsIntegrationCreate**](AwsIntegrationCreate.md) |  | [required] |

### Return type

[**crate::models::AwsIntegration**](AwsIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_aws_destroy

> integrations_aws_destroy(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_aws_list

> crate::models::PaginatedAwsIntegrationList integrations_aws_list(aws_account_id, aws_role_name, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aws_account_id** | Option<**String**> |  |  |
**aws_role_name** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedAwsIntegrationList**](PaginatedAwsIntegrationList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_aws_partial_update

> crate::models::AwsIntegration integrations_aws_partial_update(id, patched_aws_integration)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**patched_aws_integration** | Option<[**PatchedAwsIntegration**](PatchedAwsIntegration.md)> |  |  |

### Return type

[**crate::models::AwsIntegration**](AwsIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_aws_retrieve

> crate::models::AwsIntegration integrations_aws_retrieve(id, refresh_status)
Get details of an AWS Integration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**refresh_status** | Option<**bool**> | Refresh the integration status before returning the details. |  |

### Return type

[**crate::models::AwsIntegration**](AwsIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_aws_update

> crate::models::AwsIntegration integrations_aws_update(id, aws_integration)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**aws_integration** | [**AwsIntegration**](AwsIntegration.md) |  | [required] |

### Return type

[**crate::models::AwsIntegration**](AwsIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_explore_list

> crate::models::PaginatedIntegrationExplorerList integrations_explore_list(fqn, page)
Retrieve third-party integration data for the specified FQN.

### Description ###  Queries a third-party integration to retrieve the data specified by the FQN.  You can start exploring by not specifying an 'fqn', which will return a list of FQNs for the existing third-party integrations. Third-party integrations can be configured via the Integrations section of the web application. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fqn** | Option<**String**> | FQN (URL-like) for third-party integration. |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedIntegrationExplorerList**](PaginatedIntegrationExplorerList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_github_create

> crate::models::GitHubIntegration integrations_github_create(git_hub_integration_create)
Establishes a GitHub Integration.

### Description ###  Establishes a GitHub Integration in your CloudTruth organization.  ### Pre-Conditions ###  - The user must be an Administrator or Owner of your organization. - A GitHub Integration with the `installation_id` cannot  already exist in this organization. - The user must first install the CloudTruth GitHub Application in  their GitHub organization and obtain the `installation_id` of the  application in order to create the integration.  ### Initiating the GitHub Application Installation ###  - Go to `https://github.com/apps/cloudtruth-local/installations/new?state=<bearer_token>` - On successful installation the browser will return to  `https://app.localhost/app_setup/github`  and provide the `installation_id` in the URI. - POST to this api to verify and establish the integration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**git_hub_integration_create** | [**GitHubIntegrationCreate**](GitHubIntegrationCreate.md) |  | [required] |

### Return type

[**crate::models::GitHubIntegration**](GitHubIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_github_destroy

> integrations_github_destroy(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_github_list

> crate::models::PaginatedGitHubIntegrationList integrations_github_list(gh_organization_slug, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**gh_organization_slug** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedGitHubIntegrationList**](PaginatedGitHubIntegrationList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## integrations_github_retrieve

> crate::models::GitHubIntegration integrations_github_retrieve(id, refresh_status)
Get details of a GitHub Integration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**refresh_status** | Option<**bool**> | Refresh the integration status before returning the details. |  |

### Return type

[**crate::models::GitHubIntegration**](GitHubIntegration.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

