# test_parameter_rules_bool

create a basic parameter without a value, so the rule cannot be violated

```console
$ cloudtruth param set "param1" --value "true" --type "boolean" 
Set parameter 'param1' in project '[PROJECT]' for environment '[ENV]'.

$ cloudtruth param unset "param1" 
Removed parameter value 'param1' from project '[PROJECT]' for environment '[ENV]'.

```

see no rules

```console
$ cloudtruth param ls --rules -f csv -v
No parameter rules found in project [PROJECT]

```

negative tests for bad rule types: --max, --min, --max-len, --min-len, --regex

```console
$ cloudtruth param set param1 --max 100 --min 10 --max-len -10 --min-len -1 --regex "abc.*"
? 12
Rule create error: max rules not valid for boolean parameters
Rule create error: min rules not valid for boolean parameters
Rule create error: max-len rules not valid for boolean parameters
Rule create error: min-len rules not valid for boolean parameters
Rule create error: regex rules not valid for boolean parameters

$ cloudtruth param set param1 --min-len 10
? 12
Rule create error: min-len rules not valid for boolean parameters

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,-,,boolean,0,internal,false,

$ cloudtruth param ls --rules -f csv -v
No parameter rules found in project [PROJECT]

```

see we don't leave any parameter behind when creating a parameter with an invalid rule

```console
$ cloudtruth param delete -y "param1"
Removed parameter 'param1' from project '[PROJECT]'.

$ cloudtruth param set param1 --type boolean --value true --max 10
? 12
Rule create error: max rules not valid for boolean parameters

$ cloudtruth param ls -v -f csv
No parameters found in project [PROJECT]

```
