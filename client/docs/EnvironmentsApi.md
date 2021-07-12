# \EnvironmentsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**environments_create**](EnvironmentsApi.md#environments_create) | **post** /api/v1/environments/ | 
[**environments_destroy**](EnvironmentsApi.md#environments_destroy) | **delete** /api/v1/environments/{id}/ | 
[**environments_list**](EnvironmentsApi.md#environments_list) | **get** /api/v1/environments/ | 
[**environments_partial_update**](EnvironmentsApi.md#environments_partial_update) | **patch** /api/v1/environments/{id}/ | 
[**environments_retrieve**](EnvironmentsApi.md#environments_retrieve) | **get** /api/v1/environments/{id}/ | 
[**environments_update**](EnvironmentsApi.md#environments_update) | **put** /api/v1/environments/{id}/ | 



## environments_create

> crate::models::Environment environments_create(environment_create)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**environment_create** | [**EnvironmentCreate**](EnvironmentCreate.md) |  | [required] |

### Return type

[**crate::models::Environment**](Environment.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## environments_destroy

> environments_destroy(id)


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


## environments_list

> crate::models::PaginatedEnvironmentList environments_list(name, page, parent__name)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**parent__name** | Option<**String**> |  |  |

### Return type

[**crate::models::PaginatedEnvironmentList**](PaginatedEnvironmentList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## environments_partial_update

> crate::models::Environment environments_partial_update(id, patched_environment)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**patched_environment** | Option<[**PatchedEnvironment**](PatchedEnvironment.md)> |  |  |

### Return type

[**crate::models::Environment**](Environment.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## environments_retrieve

> crate::models::Environment environments_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

[**crate::models::Environment**](Environment.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## environments_update

> crate::models::Environment environments_update(id, environment)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**environment** | [**Environment**](Environment.md) |  | [required] |

### Return type

[**crate::models::Environment**](Environment.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

