# \DevicesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_device**](DevicesApi.md#create_device) | **POST** /api/devices/create | 
[**delete_device**](DevicesApi.md#delete_device) | **DELETE** /api/devices/{uid}/delete | 
[**get_device**](DevicesApi.md#get_device) | **GET** /api/devices/{uid}/info | 
[**get_device_logs**](DevicesApi.md#get_device_logs) | **GET** /api/devices/{uid}/logs | 
[**list_device_models**](DevicesApi.md#list_device_models) | **GET** /api/devices/models | 
[**list_devices**](DevicesApi.md#list_devices) | **GET** /api/devices/ | 
[**update_device**](DevicesApi.md#update_device) | **POST** /api/devices/{uid}/update | 



## create_device

> models::DeviceOrm create_device()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::DeviceOrm**](DeviceORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_device

> models::DeviceOrm delete_device(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::DeviceOrm**](DeviceORM.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_device

> models::DeviceOrm get_device(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::DeviceOrm**](DeviceORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_device_logs

> Vec<models::DeviceLog> get_device_logs(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**Vec<models::DeviceLog>**](DeviceLog.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_device_models

> Vec<models::DeviceModelOrm> list_device_models(page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i64**> |  |  |

### Return type

[**Vec<models::DeviceModelOrm>**](DeviceModelORM.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_devices

> Vec<models::DeviceOrm> list_devices(page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i64**> |  |  |

### Return type

[**Vec<models::DeviceOrm>**](DeviceORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_device

> models::DeviceOrm update_device(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::DeviceOrm**](DeviceORM.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

