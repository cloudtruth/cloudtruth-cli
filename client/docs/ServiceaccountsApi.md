# \ServiceaccountsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**serviceaccounts_create**](ServiceaccountsApi.md#serviceaccounts_create) | **post** /api/v1/serviceaccounts/ | Create a ServiceAccount user.
[**serviceaccounts_destroy**](ServiceaccountsApi.md#serviceaccounts_destroy) | **delete** /api/v1/serviceaccounts/{id}/ | 
[**serviceaccounts_list**](ServiceaccountsApi.md#serviceaccounts_list) | **get** /api/v1/serviceaccounts/ | 
[**serviceaccounts_partial_update**](ServiceaccountsApi.md#serviceaccounts_partial_update) | **patch** /api/v1/serviceaccounts/{id}/ | 
[**serviceaccounts_retrieve**](ServiceaccountsApi.md#serviceaccounts_retrieve) | **get** /api/v1/serviceaccounts/{id}/ | 
[**serviceaccounts_update**](ServiceaccountsApi.md#serviceaccounts_update) | **put** /api/v1/serviceaccounts/{id}/ | 



## serviceaccounts_create

> crate::models::ServiceAccountCreateResponse serviceaccounts_create(service_account_create_request)
Create a ServiceAccount user.

             Creates a new ServiceAccount.  A ServiceAccount is a user record intended             for machine use (such as a build system).  It does not have a username/password             but is instead accessed using an API key.              On creation, the API key will be returned.  This key will only be shown once,             is not stored on any CloudTruth system, and should be treated as a secret.  Should             the key be lost, you will need to delete and recreate the ServiceAccount in order             to generate a new API key.             

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service_account_create_request** | [**ServiceAccountCreateRequest**](ServiceAccountCreateRequest.md) |  | [required] |

### Return type

[**crate::models::ServiceAccountCreateResponse**](ServiceAccountCreateResponse.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## serviceaccounts_destroy

> serviceaccounts_destroy(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A unique value identifying this service account. | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## serviceaccounts_list

> crate::models::PaginatedServiceAccountList serviceaccounts_list(page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedServiceAccountList**](PaginatedServiceAccountList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## serviceaccounts_partial_update

> crate::models::ServiceAccount serviceaccounts_partial_update(id, patched_service_account)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A unique value identifying this service account. | [required] |
**patched_service_account** | Option<[**PatchedServiceAccount**](PatchedServiceAccount.md)> |  |  |

### Return type

[**crate::models::ServiceAccount**](ServiceAccount.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## serviceaccounts_retrieve

> crate::models::ServiceAccount serviceaccounts_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A unique value identifying this service account. | [required] |

### Return type

[**crate::models::ServiceAccount**](ServiceAccount.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## serviceaccounts_update

> crate::models::ServiceAccount serviceaccounts_update(id, service_account)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | A unique value identifying this service account. | [required] |
**service_account** | Option<[**ServiceAccount**](ServiceAccount.md)> |  |  |

### Return type

[**crate::models::ServiceAccount**](ServiceAccount.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

