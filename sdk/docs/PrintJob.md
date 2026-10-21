# PrintJob

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** | Stable job identifier | 
**user** | **uuid::Uuid** | Owning user | 
**queue** | **uuid::Uuid** | Target queue (capability-based) | 
**name** | **String** | Original filename or description | 
**asset** | **uuid::Uuid** | the file being printed (PDF, SVG, STL, OBJ, GCODE, etc.) | 
**estimated_cost** | Option<**f64**> | Estimated cost (computed at submission) | [optional]
**created_at** | **i64** | Job creation timestamp | 
**state** | **String** | Terminal lifecycle state | 
**copies** | Option<**i32**> |  | [optional]
**priority** | Option<**String**> |  | [optional]
**material** | Option<**String**> |  | [optional]
**nozzle_temp** | Option<**i32**> |  | [optional]
**bed_temp** | Option<**i32**> |  | [optional]
**layer_height** | Option<**f32**> |  | [optional]
**infill** | Option<**i32**> |  | [optional]
**supports** | Option<**String**> |  | [optional]
**notes** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


