# DeviceStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | 
**bed_temp** | **f64** | Current measured temperature of the heated bed in degrees Celsius. | 
**target_bed_temp** | Option<**f64**> | Target temperature of the heated bed, if active. | [optional]
**nozzle_temp** | **f64** | Current measured temperature of the print head nozzle in degrees Celsius. | 
**target_nozzle_temp** | Option<**f64**> | Target temperature of the print head nozzle, if active. | [optional]
**chamber_temp** | Option<**f64**> | Current measured temperature of the print chamber (if equipped). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


