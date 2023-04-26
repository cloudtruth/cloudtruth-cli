# test_parameter_names

check that there are no parameters

```console
$ cloudtruth param list -vsf csv
No parameters found in project [PROJECT]

```

create the initial parameter

```console
$ cloudtruth param set "simple_underscore" --value "something" 
Set parameter 'simple_underscore' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param get "simple_underscore" 
something

```

rename it

```console
$ cloudtruth param set -r "foo" "simple_underscore"
Updated parameter 'foo' in project '[PROJECT]'.

$ cloudtruth param get "foo" 
something

```

back to the original name

```console
$ cloudtruth param set -r "simple_underscore" "foo"
Updated parameter 'simple_underscore' in project '[PROJECT]'.

$ cloudtruth param get "simple_underscore" 
something

$ cloudtruth param delete -y "simple_underscore"
Removed parameter 'simple_underscore' from project '[PROJECT]'.

```

create the initial parameter

```console
$ cloudtruth param set "simple.dot" --value "something" 
Set parameter 'simple.dot' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param get "simple.dot" 
something

```

rename it

```console
$ cloudtruth param set -r "foo" "simple.dot"
Updated parameter 'foo' in project '[PROJECT]'.

$ cloudtruth param get "foo" 
something

```

back to the original name

```console
$ cloudtruth param set -r "simple.dot" "foo"
Updated parameter 'simple.dot' in project '[PROJECT]'.

$ cloudtruth param get "simple.dot" 
something

$ cloudtruth param delete -y "simple.dot"
Removed parameter 'simple.dot' from project '[PROJECT]'.

```

create the initial parameter

```console
$ cloudtruth param set "simple/slash" --value "something" 
Set parameter 'simple/slash' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param get "simple/slash" 
something

```

rename it

```console
$ cloudtruth param set -r "foo" "simple/slash"
Updated parameter 'foo' in project '[PROJECT]'.

$ cloudtruth param get "foo" 
something

```

back to the original name

```console
$ cloudtruth param set -r "simple/slash" "foo"
Updated parameter 'simple/slash' in project '[PROJECT]'.

$ cloudtruth param get "simple/slash" 
something

$ cloudtruth param delete -y "simple/slash"
Removed parameter 'simple/slash' from project '[PROJECT]'.

```

create the initial parameter

```console
$ cloudtruth param set "simple space" --value "something" 
Set parameter 'simple space' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param get "simple space" 
something

```

rename it

```console
$ cloudtruth param set -r "foo" "simple space"
Updated parameter 'foo' in project '[PROJECT]'.

$ cloudtruth param get "foo" 
something

```

back to the original name

```console
$ cloudtruth param set -r "simple space" "foo"
Updated parameter 'simple space' in project '[PROJECT]'.

$ cloudtruth param get "simple space" 
something

$ cloudtruth param delete -y "simple space"
Removed parameter 'simple space' from project '[PROJECT]'.

```

create the initial parameter

```console
$ cloudtruth param set "MixCase" --value "something" 
Set parameter 'MixCase' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param get "MixCase" 
something

```

rename it

```console
$ cloudtruth param set -r "foo" "MixCase"
Updated parameter 'foo' in project '[PROJECT]'.

$ cloudtruth param get "foo" 
something

```

back to the original name

```console
$ cloudtruth param set -r "MixCase" "foo"
Updated parameter 'MixCase' in project '[PROJECT]'.

$ cloudtruth param get "MixCase" 
something

$ cloudtruth param delete -y "MixCase"
Removed parameter 'MixCase' from project '[PROJECT]'.

```
