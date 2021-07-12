# \UsersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**users_destroy**](UsersApi.md#users_destroy) | **delete** /api/v1/users/{id}/ | Delete the specified user.
[**users_list**](UsersApi.md#users_list) | **get** /api/v1/users/ | 
[**users_retrieve**](UsersApi.md#users_retrieve) | **get** /api/v1/users/{id}/ | 



## users_destroy

> users_destroy(id)
Delete the specified user.

### Description ###  Delete the specified user.  This removes all access the User may have to any Organization.  ### Pre-Conditions ###  - The user cannot be the only owner of any Organization. - The bearer token must belong to the user being deleted. - All of the memberships related to the User will be deleted, so all the membership deletion pre-conditions must also be met. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_list

> crate::models::PaginatedUserList users_list(page, _type)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**_type** | Option<**String**> |  |  |

### Return type

[**crate::models::PaginatedUserList**](PaginatedUserList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_retrieve

> crate::models::User users_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

[**crate::models::User**](User.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

