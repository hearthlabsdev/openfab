# DeviceOrm

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** | Stable internal identifier | 
**serial** | **String** |  | 
**queue** | Option<**uuid::Uuid**> |  | [optional]
**name** | **String** | Human-readable name (from IPP or admin override) | 
**first_seen** | **i64** | When this printer was first observed | 
**last_seen** | **i64** | Last successful discovery or IPP query | 
**upload_dir** | Option<**String**> | upload directory where gcode is stored | [optional]
**status** | **String** | Operational status | 
**enabled** | **bool** | Whether this printer is eligible for job dispatch | 
**driver_id** | **String** |  | 
**driver_version** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


