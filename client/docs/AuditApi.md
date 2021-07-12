# \AuditApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**audit_list**](AuditApi.md#audit_list) | **get** /api/v1/audit/ | 
[**audit_retrieve**](AuditApi.md#audit_retrieve) | **get** /api/v1/audit/{id}/ | 
[**audit_summary_retrieve**](AuditApi.md#audit_summary_retrieve) | **get** /api/v1/audit/summary/ | 



## audit_list

> crate::models::PaginatedAuditTrailList audit_list(action, earliest, latest, object_id, object_type, page, user_id)


A searchable log of all the actions taken by users and service accounts within the organization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**action** | Option<**String**> | The action that was taken. |  |
**earliest** | Option<**String**> |  |  |
**latest** | Option<**String**> |  |  |
**object_id** | Option<**String**> |  |  |
**object_type** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**user_id** | Option<**String**> |  |  |

### Return type

[**crate::models::PaginatedAuditTrailList**](PaginatedAuditTrailList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## audit_retrieve

> crate::models::AuditTrail audit_retrieve(id)


Retrieve one record from the audit log.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

[**crate::models::AuditTrail**](AuditTrail.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## audit_summary_retrieve

> crate::models::AuditTrailSummary audit_summary_retrieve()


Summary information about the organization's audit trail.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AuditTrailSummary**](AuditTrailSummary.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

