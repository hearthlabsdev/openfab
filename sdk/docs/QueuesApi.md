# \QueuesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_prints**](QueuesApi.md#list_prints) | **GET** /api/prints/queues/{uid}/jobs | 
[**list_queues**](QueuesApi.md#list_queues) | **GET** /api/prints/queues | 
[**queue_info**](QueuesApi.md#queue_info) | **GET** /api/prints/queues/{uid}/info | 



## list_prints

> Vec<models::PrintJob> list_prints(uid)


lists the prints in the given queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**Vec<models::PrintJob>**](PrintJob.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_queues

> Vec<models::PrintQueueOrm> list_queues()


lists queues on the server

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::PrintQueueOrm>**](PrintQueueORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## queue_info

> models::PrintQueueOrm queue_info(uid)


get info on a specific Queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::PrintQueueOrm**](PrintQueueORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

