# Parameter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**url** | **String** |  | [readonly]
**id** | **String** | A unique identifier for the parameter. | [readonly]
**name** | **String** | The parameter name. | 
**description** | Option<**String**> | A description of the parameter.  You may find it helpful to document how this parameter is used to assist others when they need to maintain software that uses this content. | [optional]
**secret** | Option<**bool**> | Indicates if this content is secret or not.  When a parameter is considered to be a secret, any static values are stored in a dedicated vault for your organization for maximum security.  Dynamic values are inspected on-demand to ensure they align with the parameter's secret setting and if they do not, those dynamic values are not allowed to be used. | [optional]
**templates** | **Vec<String>** |  | [readonly]
**uses_dynamic_values** | **bool** | If `true` then at least one of the values is dynamic. | [readonly]
**values** | [**::std::collections::HashMap<String, crate::models::Value>**](Value.md) |              Each parameter has an effective value in every environment based on             environment inheritance and which environments have had a value set.              Environments inherit from a single parent to form a tree, as a result             a single parameter may have different values present for each environment.             When a value is not explicitly set in an environment, the parent environment             is consulted to see if it has a value defined, and so on.              The dictionary of values has an environment url as the key, and the optional             Value record that it resolves to.  If the Value.environment matches the key,             then it is an explicit value set for that environment.  If they differ, the             value was obtained from a parent environment (directly or indirectly).  If the             value is None then no value has ever been set in any environment for this             parameter.              key: Environment url             value: optional Value record          | [readonly]
**created_at** | **String** |  | [readonly]
**modified_at** | **String** |  | [readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


