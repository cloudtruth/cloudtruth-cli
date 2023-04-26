# test_parameter_export

check that there are no parameters -- to avoid later confusion

```console
$ cloudtruth parameters list
No parameters found in project [PROJECT]

```

add first, non-secret parameter

```console
$ cloudtruth param set "first_param" --value "posix_compliant_value" 
Set parameter 'first_param' in project '[PROJECT]' for environment 'default'.

```

add first, non-secret parameter

```console
$ cloudtruth param set "SECOND_PARAM" --value "a value with spaces" 
Set parameter 'SECOND_PARAM' in project '[PROJECT]' for environment 'default'.

```

add a non-posix complaint key with a posix value

```console
$ cloudtruth param set "non.posix.key" --value "posix_value_invalid_key" 
Set parameter 'non.posix.key' in project '[PROJECT]' for environment 'default'.

```

add first, secret parameter

```console
$ cloudtruth param set "FIRST_PARAM_SECRET" --value "top-secret-sci" --secret "true" 
Set parameter 'FIRST_PARAM_SECRET' in project '[PROJECT]' for environment 'default'.

```

add first, secret parameter

```console
$ cloudtruth param set "second_secret" --value "sensitive value with spaces" --secret "true" 
Set parameter 'second_secret' in project '[PROJECT]' for environment 'default'.

```

Docker

```console
$ cloudtruth param export docker 
*****

$ cloudtruth param export docker --secrets
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces


$ cloudtruth param export docker --secrets --starts-with SECOND
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces


```

use uppercase key without secrets

```console
$ cloudtruth param export docker --starts-with FIRST
*****

```

use uppercase key with secrets

```console
$ cloudtruth param export docker --starts-with FIRST -s
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci


```

use lowercase key with secrets

```console
$ cloudtruth param export docker --contains param -s
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces


```

see if filter picks up non-posix

```console
$ cloudtruth param export docker --contains posix -s


```

Dotenv

```console
$ cloudtruth param export dotenv 
*****

$ cloudtruth param export dotenv -s
FIRST_PARAM="posix_compliant_value"
FIRST_PARAM_SECRET="top-secret-sci"
SECOND_PARAM="a value with spaces"
SECOND_SECRET="sensitive value with spaces"


```

Shell

```console
$ cloudtruth param export shell 
*****

$ cloudtruth param export shell -s
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM='a value with spaces'
SECOND_SECRET='sensitive value with spaces'


```
