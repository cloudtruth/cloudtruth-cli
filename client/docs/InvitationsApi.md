# \InvitationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**invitations_accept_create**](InvitationsApi.md#invitations_accept_create) | **post** /api/v1/invitations/{id}/accept/ | Accept an invitation.
[**invitations_create**](InvitationsApi.md#invitations_create) | **post** /api/v1/invitations/ | Create an invitation.
[**invitations_destroy**](InvitationsApi.md#invitations_destroy) | **delete** /api/v1/invitations/{id}/ | 
[**invitations_list**](InvitationsApi.md#invitations_list) | **get** /api/v1/invitations/ | 
[**invitations_partial_update**](InvitationsApi.md#invitations_partial_update) | **patch** /api/v1/invitations/{id}/ | 
[**invitations_resend_create**](InvitationsApi.md#invitations_resend_create) | **post** /api/v1/invitations/{id}/resend/ | Resend an invitation.
[**invitations_retrieve**](InvitationsApi.md#invitations_retrieve) | **get** /api/v1/invitations/{id}/ | 
[**invitations_update**](InvitationsApi.md#invitations_update) | **put** /api/v1/invitations/{id}/ | 



## invitations_accept_create

> crate::models::Invitation invitations_accept_create(id)
Accept an invitation.

Accept an invitation to join an organization.  The email address used to log in and accept the invitation must match the email address specified by the inviting user when creating the invitation.  On success the client receives the invitation record as it was updated. The client should then regenerate the JWT with the organization scope and proceed to the default landing page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) | The invitation ID. | [required] |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_create

> crate::models::Invitation invitations_create(invitation_create)
Create an invitation.

Extend an invitation for someone else to join your organization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invitation_create** | [**InvitationCreate**](InvitationCreate.md) |  | [required] |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_destroy

> invitations_destroy(id)


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


## invitations_list

> crate::models::PaginatedInvitationList invitations_list(email, page, role, state)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**email** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**role** | Option<**String**> | The role that the user will have in the organization, should the user accept. |  |
**state** | Option<**String**> | The current state of the invitation. |  |

### Return type

[**crate::models::PaginatedInvitationList**](PaginatedInvitationList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_partial_update

> crate::models::Invitation invitations_partial_update(id, patched_invitation)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**patched_invitation** | Option<[**PatchedInvitation**](PatchedInvitation.md)> |  |  |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_resend_create

> crate::models::Invitation invitations_resend_create(id)
Resend an invitation.

Re-send an invitation to the recipient.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) | The invitation ID. | [required] |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_retrieve

> crate::models::Invitation invitations_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invitations_update

> crate::models::Invitation invitations_update(id, invitation)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**invitation** | [**Invitation**](Invitation.md) |  | [required] |

### Return type

[**crate::models::Invitation**](Invitation.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

