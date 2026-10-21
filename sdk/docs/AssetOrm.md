# AssetOrm

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **uuid::Uuid** |  | 
**name** | **String** |  | 
**library** | **uuid::Uuid** |  | 
**mime** | **String** |  | 
**key** | **String** |  | 
**owner** | **uuid::Uuid** | Owning user (for permissions and sharing) | 
**hash_alg** | **String** |  | 
**hash** | **String** |  | 
**size** | **i64** | Size of the file in bytes. | 
**created_at** | **i64** |  | 
**updated_at** | **i64** | when the file was last modified | 
**last_used_at** | Option<**i64**> | Last time this asset was used in a print job or queue. | [optional]
**public** | **bool** | Whether this asset is publicly accessible (e.g. for embedding or sharing) | 
**license** | Option<**String**> | Optional license or usage terms (e.g. \"CC-BY-SA 4.0\") | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


