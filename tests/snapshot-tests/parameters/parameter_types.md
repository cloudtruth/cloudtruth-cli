
# test_parameter_types

boolean tests

```console
$ cloudtruth param set param1 -t boolean -v true
Set parameter 'param1' in project '[PROJECT]' for environment 'default'.

```

see it in the display

```console
$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,

```

try to set value to non-bool value

```console
$ cloudtruth param set param1 -v not-a-bool
? 1
Error: 
   0: [91mRule violation: Value is not of type boolean[0m
...
```

change the type back to string

```console
$ cloudtruth param set "param1" --value "true" --type "string" 
Updated parameter 'param1' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,string,0,internal,false,

```

update back to bool

```console
$ cloudtruth param set "param1" --value "true" --type "boolean" 
Updated parameter 'param1' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,

```

integer tests

```console
$ cloudtruth param set param2 -t integer -v -1234
Set parameter 'param2' in project '[PROJECT]' for environment 'default'.

```

see it in the display

```console
$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,
param2,-1234,default,integer,0,internal,false,

```

try to set value to non-integer value

```console
$ cloudtruth param set param2 -v not-an-integer
? 1
Error: 
   0: [91mRule violation: Value is not of type integer[0m
...
```

change the type back to string

```console
$ cloudtruth param set "param2" --value "-1234" --type "string" 
Updated parameter 'param2' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,
param2,-1234,default,string,0,internal,false,

```

update back to integer

```console
$ cloudtruth param set "param2" --value "-1234" --type "integer" 
Updated parameter 'param2' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,
param2,-1234,default,integer,0,internal,false,

```

NOTE: no real need to test 'string' types, since that is the default and no illegal values

```console
$ cloudtruth param set param3 --type foo
? 1
Error: 
   0: [91mNot Found (404): No ParameterType matches the given query.[0m
...

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,true,default,boolean,0,internal,false,
param2,-1234,default,integer,0,internal,false,

```
