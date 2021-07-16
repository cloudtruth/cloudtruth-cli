# \MembershipsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**memberships_create**](MembershipsApi.md#memberships_create) | **post** /api/v1/memberships/ | 
[**memberships_destroy**](MembershipsApi.md#memberships_destroy) | **delete** /api/v1/memberships/{id}/ | 
[**memberships_list**](MembershipsApi.md#memberships_list) | **get** /api/v1/memberships/ | 
[**memberships_partial_update**](MembershipsApi.md#memberships_partial_update) | **patch** /api/v1/memberships/{id}/ | 
[**memberships_retrieve**](MembershipsApi.md#memberships_retrieve) | **get** /api/v1/memberships/{id}/ | 
[**memberships_update**](MembershipsApi.md#memberships_update) | **put** /api/v1/memberships/{id}/ | 



## memberships_create

> crate::models::Membership memberships_create(membership_create)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**membership_create** | [**MembershipCreate**](MembershipCreate.md) |  | [required] |

### Return type

[**crate::models::Membership**](Membership.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## memberships_destroy

> memberships_destroy(id)


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


## memberships_list

> crate::models::PaginatedMembershipList memberships_list(page, role, user)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**role** | Option<**String**> | The role that the user has in the organization. |  |
**user** | Option<**String**> | The unique identifier of a user. |  |

### Return type

[**crate::models::PaginatedMembershipList**](PaginatedMembershipList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## memberships_partial_update

> crate::models::Membership memberships_partial_update(id, patched_membership)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**patched_membership** | Option<[**PatchedMembership**](PatchedMembership.md)> |  |  |

### Return type

[**crate::models::Membership**](Membership.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## memberships_retrieve

> crate::models::Membership memberships_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

[**crate::models::Membership**](Membership.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## memberships_update

> crate::models::Membership memberships_update(id, membership)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**membership** | [**Membership**](Membership.md) |  | [required] |

### Return type

[**crate::models::Membership**](Membership.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

