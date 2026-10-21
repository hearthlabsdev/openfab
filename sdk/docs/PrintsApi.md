# \PrintsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_print**](PrintsApi.md#create_print) | **POST** /api/prints/create | 
[**delete_print**](PrintsApi.md#delete_print) | **DELETE** /api/prints/jobs/{uid}/delete | 
[**get_print**](PrintsApi.md#get_print) | **GET** /api/prints/jobs/{uid}/info | 
[**get_print_status**](PrintsApi.md#get_print_status) | **GET** /api/prints/jobs/{uid}/status | 
[**list_instances**](PrintsApi.md#list_instances) | **GET** /api/prints/jobs/{uid}/instances | 
[**pause_print**](PrintsApi.md#pause_print) | **PUT** /api/prints/jobs/{uid}/pause | 
[**start_print**](PrintsApi.md#start_print) | **POST** /api/prints/jobs/{uid}/start | 
[**stop_print**](PrintsApi.md#stop_print) | **PUT** /api/prints/jobs/{uid}/stop | 



## create_print

> models::PrintJob create_print()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::PrintJob**](PrintJob.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_print

> models::PrintJob delete_print(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PrintJob**](PrintJob.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_print

> models::PrintJob get_print(uid)


gets information about a print

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PrintJob**](PrintJob.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_print_status

> models::PrintJobInstance get_print_status(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PrintJobInstance**](PrintJobInstance.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_instances

> Vec<models::PrintJobInstance> list_instances(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**Vec<models::PrintJobInstance>**](PrintJobInstance.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pause_print

> pause_print(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_print

> models::PrintJobInstance start_print(uid, device)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |
**device** | Option<**uuid::Uuid**> |  |  |

### Return type

[**models::PrintJobInstance**](PrintJobInstance.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_print

> models::PrintJobInstance stop_print(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PrintJobInstance**](PrintJobInstance.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

