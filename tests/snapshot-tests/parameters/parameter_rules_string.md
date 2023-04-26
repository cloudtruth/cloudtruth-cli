# test_parameter_rules_strings

create a basic parameter without a value, so the rule cannot be violated

```console
$ cloudtruth param set "param1" --value "some-value" 
Set parameter 'param1' in project '[PROJECT]' for environment '[ENV]'.

$ cloudtruth param unset "param1" 
Removed parameter value 'param1' from project '[PROJECT]' for environment '[ENV]'.

```

see no rules

```console
$ cloudtruth param ls --rules -v -f csv
No parameter rules found in project [PROJECT]

$ cloudtruth param set param1 --min-len 10 --max-len 15 --regex "abc.*"
Updated parameter 'param1' in project '[PROJECT]'.

```

see the 3 rules are registered

```console
$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,-,,string,3,internal,false,

```

check the --rules output (csv)

```console
$ cloudtruth param ls --rules -v -f csv
Name,Param Type,Rule Type,Constraint
param1,string,max-len,15
param1,string,min-len,10
param1,string,regex,abc.*

$ cloudtruth param list --rules
param1

$ cloudtruth param list --rules -v
+--------+------------+-----------+------------+
| Name   | Param Type | Rule Type | Constraint |
+--------+------------+-----------+------------+
| param1 | string     | max-len   | 15         |
| param1 | string     | min-len   | 10         |
| param1 | string     | regex     | abc.*      |
+--------+------------+-----------+------------+

```

test min-len

```console
$ cloudtruth param set param1 -v aaaaaaaaa
? 1
Error: 
   0: [91mRule violation: Value must be at least 10 characters[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

test max-len

```console
$ cloudtruth param set param1 -v aaaaaaaaaaaaaaaa
? 1
Error: 
   0: [91mRule violation: Value must be at most 15 characters[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

test regex

```console
$ cloudtruth param set param1 -v aaaaaaaaaaaaaaa
? 1
Error: 
   0: [91mRule violation: Value does not match regular expression abc.*[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

something in the middle, so it is successful

```console
$ cloudtruth param set param1 -v abcabcabcabc
Set parameter 'param1' in project '[PROJECT]' for environment '[ENV]'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,3,internal,false,

```

update the rules

```console
$ cloudtruth param set param1 --min-len 5
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param set param1 --max-len 30
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param set param1 --regex "a.*"
Updated parameter 'param1' in project '[PROJECT]'.

```

see the 3 rules are registered

```console
$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,3,internal,false,

```

check the --rules output (csv)

```console
$ cloudtruth param ls --rules -v -f csv
Name,Param Type,Rule Type,Constraint
param1,string,max-len,30
param1,string,min-len,5
param1,string,regex,a.*

```

remove the rules, one by one

```console
$ cloudtruth param set param1 --no-regex
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,2,internal,false,

$ cloudtruth param ls --rules -v -f csv
Name,Param Type,Rule Type,Constraint
param1,string,max-len,30
param1,string,min-len,5

$ cloudtruth param set param1 --no-regex
Updated parameter 'param1' in project '[PROJECT]'.

```

max-len

```console
$ cloudtruth param set param1 --no-max-len
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,1,internal,false,

$ cloudtruth param ls --rules -v -f csv
Name,Param Type,Rule Type,Constraint
param1,string,min-len,5

$ cloudtruth param set param1 --no-max-len
Updated parameter 'param1' in project '[PROJECT]'.

```

min-len

```console
$ cloudtruth param set param1 --no-min-len
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,0,internal,false,

$ cloudtruth param ls --rules -v -f csv
No parameter rules found in project [PROJECT]

$ cloudtruth param set param1 --no-min-len
Updated parameter 'param1' in project '[PROJECT]'.

```

failed create rules with values in place

```console
$ cloudtruth param set param1 --min-len 15
? 12
Rule create error: Rule may not be applied to param1: [ENV]: ['Value must be at least 15 characters']

$ cloudtruth param set param1 --max-len 10
? 12
Rule create error: Rule may not be applied to param1: [ENV]: ['Value must be at most 10 characters']

$ cloudtruth param set param1 --min-len 2
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param set param1 --min-len 15
? 12
Rule update error: Rule may not be applied to param1: [ENV]: ['Value must be at least 15 characters']

$ cloudtruth param set param1 --max-len 22
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param set param1 --max-len 10
? 12
Rule update error: Rule may not be applied to param1: [ENV]: ['Value must be at most 10 characters']

```

remove the rules

```console
$ cloudtruth param set param1 --no-min-len
Updated parameter 'param1' in project '[PROJECT]'.

$ cloudtruth param set param1 --no-max-len
Updated parameter 'param1' in project '[PROJECT]'.

```

negative tests for bad rule types: --max, and --min

```console
$ cloudtruth param set param1 --max -10 --min -1
? 12
Rule create error: max rules not valid for string parameters
Rule create error: min rules not valid for string parameters

$ cloudtruth param set param1 --max -10
? 12
Rule create error: max rules not valid for string parameters

$ cloudtruth param ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
param1,abcabcabcabc,[ENV],string,0,internal,false,

$ cloudtruth param ls --rules -v -f csv
No parameter rules found in project [PROJECT]

```

see we don't leave any parameter behind when creating a parameter with an invalid rule

```console
$ cloudtruth param delete -y "param1"
Removed parameter 'param1' from project '[PROJECT]'.

$ cloudtruth param set param1 --type string --value 9 --max 10
? 12
Rule create error: max rules not valid for string parameters

$ cloudtruth param ls -v -f csv
No parameters found in project [PROJECT]

```