# PrintJobInstance

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | 
**job** | [**models::PrintJob**](PrintJob.md) |  | 
**printer** | [**models::DeviceOrm**](DeviceORM.md) |  | 
**instance** | **i32** |  | 
**events** | [**Vec<models::PrintJobEvent>**](PrintJobEvent.md) |  | 
**started_at** | Option<**i64**> |  | [optional]
**finished_at** | Option<**i64**> |  | [optional]
**error** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


