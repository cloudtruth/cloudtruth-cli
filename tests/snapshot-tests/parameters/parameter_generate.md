# test_parameter_generate

add a project

```console
$ cloudtruth param set param1 --generate
Set parameter 'param1' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param set param2 --generate --secret true
Set parameter 'param2' in project '[PROJECT]' for environment 'default'.

```

see secrets are secret, but generated values do not need to be secret


```console
$ cloudtruth param ls -sf json
{
  "parameter": [
    {
      "Description": "",
      "Name": "param1",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "[..]"
    },
    {
      "Description": "",
      "Name": "param2",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "internal",
      "Value": "[..]"
    }
  ]
}

$ cloudtruth param set param1 --generate
Updated parameter 'param1' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param set param2 --generate
Updated parameter 'param2' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param ls -sf json
{
  "parameter": [
    {
      "Description": "",
      "Name": "param1",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "[..]"
    },
    {
      "Description": "",
      "Name": "param2",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "internal",
      "Value": "[..]"
    }
  ]
}

```

does not work with boolean/integer types

```console
$ cloudtruth param set "param3" --value "true" --type "boolean" 
Set parameter 'param3' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param set "param4" --value "123456" --type "integer" 
Set parameter 'param4' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param set param3 --generate
? 1
Error: 
   0: [91mRule violation: Value is not of type boolean[0m
...

$ cloudtruth param set param4 --generate
? 1
Error: 
   0: [91mRule violation: Value is not of type integer[0m
...

```

does not work with rules... should possibly change next iteration

```console
$ cloudtruth param set param5 --min-len 50 --generate
? 1
Error: 
   0: [91mRule violation: Value must be at least 50 characters[0m
...

```
