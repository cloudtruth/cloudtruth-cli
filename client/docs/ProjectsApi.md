# \ProjectsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**projects_create**](ProjectsApi.md#projects_create) | **post** /api/v1/projects/ | 
[**projects_destroy**](ProjectsApi.md#projects_destroy) | **delete** /api/v1/projects/{id}/ | 
[**projects_list**](ProjectsApi.md#projects_list) | **get** /api/v1/projects/ | 
[**projects_parameter_export_list**](ProjectsApi.md#projects_parameter_export_list) | **get** /api/v1/projects/{project_pk}/parameter-export/ | 
[**projects_parameters_create**](ProjectsApi.md#projects_parameters_create) | **post** /api/v1/projects/{project_pk}/parameters/ | 
[**projects_parameters_destroy**](ProjectsApi.md#projects_parameters_destroy) | **delete** /api/v1/projects/{project_pk}/parameters/{id}/ | 
[**projects_parameters_list**](ProjectsApi.md#projects_parameters_list) | **get** /api/v1/projects/{project_pk}/parameters/ | 
[**projects_parameters_partial_update**](ProjectsApi.md#projects_parameters_partial_update) | **patch** /api/v1/projects/{project_pk}/parameters/{id}/ | 
[**projects_parameters_retrieve**](ProjectsApi.md#projects_parameters_retrieve) | **get** /api/v1/projects/{project_pk}/parameters/{id}/ | 
[**projects_parameters_update**](ProjectsApi.md#projects_parameters_update) | **put** /api/v1/projects/{project_pk}/parameters/{id}/ | 
[**projects_parameters_values_create**](ProjectsApi.md#projects_parameters_values_create) | **post** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/ | Set a value.
[**projects_parameters_values_destroy**](ProjectsApi.md#projects_parameters_values_destroy) | **delete** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/{id}/ | Destroy a value.
[**projects_parameters_values_list**](ProjectsApi.md#projects_parameters_values_list) | **get** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/ | Retrieve values.
[**projects_parameters_values_partial_update**](ProjectsApi.md#projects_parameters_values_partial_update) | **patch** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/{id}/ | Update a value.
[**projects_parameters_values_retrieve**](ProjectsApi.md#projects_parameters_values_retrieve) | **get** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/{id}/ | Retrieve a value.
[**projects_parameters_values_update**](ProjectsApi.md#projects_parameters_values_update) | **put** /api/v1/projects/{project_pk}/parameters/{parameter_pk}/values/{id}/ | Update a value.
[**projects_partial_update**](ProjectsApi.md#projects_partial_update) | **patch** /api/v1/projects/{id}/ | 
[**projects_retrieve**](ProjectsApi.md#projects_retrieve) | **get** /api/v1/projects/{id}/ | 
[**projects_template_preview_create**](ProjectsApi.md#projects_template_preview_create) | **post** /api/v1/projects/{project_pk}/template-preview/ | 
[**projects_templates_create**](ProjectsApi.md#projects_templates_create) | **post** /api/v1/projects/{project_pk}/templates/ | 
[**projects_templates_destroy**](ProjectsApi.md#projects_templates_destroy) | **delete** /api/v1/projects/{project_pk}/templates/{id}/ | 
[**projects_templates_list**](ProjectsApi.md#projects_templates_list) | **get** /api/v1/projects/{project_pk}/templates/ | 
[**projects_templates_partial_update**](ProjectsApi.md#projects_templates_partial_update) | **patch** /api/v1/projects/{project_pk}/templates/{id}/ | 
[**projects_templates_retrieve**](ProjectsApi.md#projects_templates_retrieve) | **get** /api/v1/projects/{project_pk}/templates/{id}/ | 
[**projects_templates_update**](ProjectsApi.md#projects_templates_update) | **put** /api/v1/projects/{project_pk}/templates/{id}/ | 
[**projects_update**](ProjectsApi.md#projects_update) | **put** /api/v1/projects/{id}/ | 



## projects_create

> crate::models::Project projects_create(project_create)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_create** | [**ProjectCreate**](ProjectCreate.md) |  | [required] |

### Return type

[**crate::models::Project**](Project.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_destroy

> projects_destroy(id)


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


## projects_list

> crate::models::PaginatedProjectList projects_list(name, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedProjectList**](PaginatedProjectList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameter_export_list

> crate::models::ParameterExport projects_parameter_export_list(project_pk, contains, endswith, environment, explicit_export, mask_secrets, output, page, startswith, wrap)


Exports all parameters in this project in the requested format.  Parameter names and values will be coerced to the proper format (e.g. for a dotenv export, my_parameter will be capitalized to MY_PARAMETER and its value will be in a quoted string).  Note that capitalization is the only name coercion that will be performed on parameter names, names that are invalid for a given format will be omitted.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**contains** | Option<**String**> | (Optional) Only include parameters whose names contain the provided string. |  |
**endswith** | Option<**String**> | (Optional) Only include parameters whose names end with the provided string. |  |
**environment** | Option<[**String**](.md)> | (Optional) ID of the environment to use to instantiate this template |  |
**explicit_export** | Option<**bool**> | If true, explicitly marks parameters with export, e.g. export FOO=bar.  Defaults to false. |  |
**mask_secrets** | Option<**bool**> | If true, masks all secrets in the template (defaults to false) |  |
**output** | Option<**String**> | Format to output: One of 'docker', 'dotenv', 'shell'. |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**startswith** | Option<**String**> | (Optional) Only include parameters whose names start with the provided string. |  |
**wrap** | Option<**bool**> | Indicates all secrets are wrapped.  For more information on secret wrapping, see the documentation. |  |

### Return type

[**crate::models::ParameterExport**](ParameterExport.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_create

> crate::models::Parameter projects_parameters_create(project_pk, parameter_create)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**parameter_create** | [**ParameterCreate**](ParameterCreate.md) |  | [required] |

### Return type

[**crate::models::Parameter**](Parameter.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_destroy

> projects_parameters_destroy(id, project_pk)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_list

> crate::models::PaginatedParameterList projects_parameters_list(project_pk, environment, mask_secrets, name, page, wrap)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**environment** | Option<[**String**](.md)> | (Optional) ID of the environment to get parameter values for. |  |
**mask_secrets** | Option<**bool**> | If true, masks all secrets (defaults to false). |  |
**name** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**wrap** | Option<**bool**> | If true, wraps all secrets (defaults to false) - see documentation for more details. |  |

### Return type

[**crate::models::PaginatedParameterList**](PaginatedParameterList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_partial_update

> crate::models::Parameter projects_parameters_partial_update(id, project_pk, patched_parameter)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**patched_parameter** | Option<[**PatchedParameter**](PatchedParameter.md)> |  |  |

### Return type

[**crate::models::Parameter**](Parameter.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_retrieve

> crate::models::Parameter projects_parameters_retrieve(id, project_pk, environment, mask_secrets, wrap)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**environment** | Option<[**String**](.md)> | (Optional) ID of the environment to get parameter values for. |  |
**mask_secrets** | Option<**bool**> | If true, masks all secrets (defaults to false). |  |
**wrap** | Option<**bool**> | If true, wraps all secrets (defaults to false) - see documentation for more details. |  |

### Return type

[**crate::models::Parameter**](Parameter.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_update

> crate::models::Parameter projects_parameters_update(id, project_pk, parameter)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**parameter** | [**Parameter**](Parameter.md) |  | [required] |

### Return type

[**crate::models::Parameter**](Parameter.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_create

> crate::models::Value projects_parameters_values_create(parameter_pk, project_pk, value_create, wrap)
Set a value.

Set the value of a parameter in an environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |
**value_create** | [**ValueCreate**](ValueCreate.md) |  | [required] |
**wrap** | Option<**bool**> | Indicates the `static_value` is a wrapped secret. For more information on secret wrapping, see the documentation.  |  |

### Return type

[**crate::models::Value**](Value.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_destroy

> projects_parameters_values_destroy(id, parameter_pk, project_pk)
Destroy a value.

Destroy the value of a parameter in an environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_list

> crate::models::PaginatedValueList projects_parameters_values_list(parameter_pk, project_pk, environment, mask_secrets, page, wrap)
Retrieve values.

         Retrieve previously set values of a parameter in one or all environments.         To see all the _effective_ values for a parameter across every environment,         use the Parameters API (see the `values` field).     

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |
**environment** | Option<**String**> | ID of the environment to limit the result to.  If this is not specified then the result will contain a value for any environment in which it is set.  You cannot use this option to retrieve the _effective_ value of a parameter in an environment for which is is not explicitly set.  To see _effective_ values use the Parameters API (see the `values` field). |  |
**mask_secrets** | Option<**bool**> | (Optional) If true, mask secret values in responses (defaults to false). |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |
**wrap** | Option<**bool**> | For writes, indicates `static_value` is wrapped; for reads, indicates `value` is wrapped. For more information on secret wrapping, see the documentation.  |  |

### Return type

[**crate::models::PaginatedValueList**](PaginatedValueList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_partial_update

> crate::models::Value projects_parameters_values_partial_update(id, parameter_pk, project_pk, wrap, patched_value)
Update a value.

Update the value of a parameter in an environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |
**wrap** | Option<**bool**> | Indicates the `static_value` is a wrapped secret. For more information on secret wrapping, see the documentation.  |  |
**patched_value** | Option<[**PatchedValue**](PatchedValue.md)> |  |  |

### Return type

[**crate::models::Value**](Value.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_retrieve

> crate::models::Value projects_parameters_values_retrieve(id, parameter_pk, project_pk, mask_secrets, wrap)
Retrieve a value.

Retrieve the value of a parameter in an environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |
**mask_secrets** | Option<**bool**> | (Optional) If true, mask secret values in responses (defaults to false). |  |
**wrap** | Option<**bool**> | For writes, indicates `static_value` is wrapped; for reads, indicates `value` is wrapped. For more information on secret wrapping, see the documentation.  |  |

### Return type

[**crate::models::Value**](Value.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_parameters_values_update

> crate::models::Value projects_parameters_values_update(id, parameter_pk, project_pk, wrap, value)
Update a value.

Update the value of a parameter in an environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**parameter_pk** | [**String**](.md) | The parameter id. | [required] |
**project_pk** | [**String**](.md) | The project id. | [required] |
**wrap** | Option<**bool**> | Indicates the `static_value` is a wrapped secret. For more information on secret wrapping, see the documentation.  |  |
**value** | Option<[**Value**](Value.md)> |  |  |

### Return type

[**crate::models::Value**](Value.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_partial_update

> crate::models::Project projects_partial_update(id, patched_project)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**patched_project** | Option<[**PatchedProject**](PatchedProject.md)> |  |  |

### Return type

[**crate::models::Project**](Project.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_retrieve

> crate::models::Project projects_retrieve(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |

### Return type

[**crate::models::Project**](Project.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_template_preview_create

> crate::models::TemplatePreview projects_template_preview_create(project_pk, template_preview, environment, mask_secrets)


Endpoint for previewing a template.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**template_preview** | [**TemplatePreview**](TemplatePreview.md) |  | [required] |
**environment** | Option<[**String**](.md)> | (Optional) ID of the environment to use to instantiate this template |  |
**mask_secrets** | Option<**bool**> | If true, masks all secrets in the template (defaults to false) |  |

### Return type

[**crate::models::TemplatePreview**](TemplatePreview.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_create

> crate::models::TemplateCreate projects_templates_create(project_pk, template_create)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**template_create** | [**TemplateCreate**](TemplateCreate.md) |  | [required] |

### Return type

[**crate::models::TemplateCreate**](TemplateCreate.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_destroy

> projects_templates_destroy(id, project_pk)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_list

> crate::models::PaginatedTemplateList projects_templates_list(project_pk, name, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_pk** | [**String**](.md) |  | [required] |
**name** | Option<**String**> |  |  |
**page** | Option<**i32**> | A page number within the paginated result set. |  |

### Return type

[**crate::models::PaginatedTemplateList**](PaginatedTemplateList.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_partial_update

> crate::models::Template projects_templates_partial_update(id, project_pk, patched_template)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**patched_template** | Option<[**PatchedTemplate**](PatchedTemplate.md)> |  |  |

### Return type

[**crate::models::Template**](Template.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_retrieve

> crate::models::Template projects_templates_retrieve(id, project_pk, environment, mask_secrets)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**environment** | Option<[**String**](.md)> | (Optional) ID of the environment to use to instantiate this template |  |
**mask_secrets** | Option<**bool**> | If true, masks all secrets in the template (defaults to false) |  |

### Return type

[**crate::models::Template**](Template.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_templates_update

> crate::models::Template projects_templates_update(id, project_pk, template)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project_pk** | [**String**](.md) |  | [required] |
**template** | [**Template**](Template.md) |  | [required] |

### Return type

[**crate::models::Template**](Template.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## projects_update

> crate::models::Project projects_update(id, project)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**String**](.md) |  | [required] |
**project** | [**Project**](Project.md) |  | [required] |

### Return type

[**crate::models::Project**](Project.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [JWTAuth](../README.md#JWTAuth)

### HTTP request headers

- **Content-Type**: application/json, application/x-www-form-urlencoded, multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

