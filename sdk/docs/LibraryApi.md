# \LibraryApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_asset**](LibraryApi.md#create_asset) | **POST** /api/library/assets | 
[**create_library**](LibraryApi.md#create_library) | **POST** /api/library/create | 
[**delete_asset**](LibraryApi.md#delete_asset) | **DELETE** /api/library/assets/{uid}/delete | 
[**delete_library**](LibraryApi.md#delete_library) | **DELETE** /api/library/{uid}/delete | 
[**get_asset**](LibraryApi.md#get_asset) | **GET** /api/library/assets/{uid} | 
[**list_assets**](LibraryApi.md#list_assets) | **GET** /api/library/{uid}/assets | 
[**list_libraries**](LibraryApi.md#list_libraries) | **GET** /api/library/library | 
[**update_asset**](LibraryApi.md#update_asset) | **PUT** /api/library/{uid}/update | 



## create_asset

> models::AssetOrm create_asset()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AssetOrm**](AssetORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_library

> models::LibraryOrm create_library()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::LibraryOrm**](LibraryORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_asset

> models::AssetOrm delete_asset(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::AssetOrm**](AssetORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_library

> models::LibraryOrm delete_library(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::LibraryOrm**](LibraryORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_asset

> models::AssetOrm get_asset(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::AssetOrm**](AssetORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_assets

> Vec<models::AssetOrm> list_assets(uid, page)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |
**page** | Option<**i64**> |  |  |

### Return type

[**Vec<models::AssetOrm>**](AssetORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_libraries

> Vec<models::LibraryOrm> list_libraries()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::LibraryOrm>**](LibraryORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_asset

> models::AssetOrm update_asset(uid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::AssetOrm**](AssetORM.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

