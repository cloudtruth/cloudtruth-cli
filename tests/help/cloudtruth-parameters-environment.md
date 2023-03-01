```console
$ cloudtruth parameters environment --help
cloudtruth-parameters-environment 
Shows the environments with parameter overrides

USAGE:
    cloudtruth parameters environment [FLAGS] [OPTIONS] <KEY>

FLAGS:
    -a, --all           Show even unset environments.
    -h, --help          Prints help information
    -s, --secrets       Display secret values in environments
        --show-times    Show create and modified times.
    -V, --version       Prints version information

OPTIONS:
        --as-of <datetime|tag>    Date/time (or tag) of parameter value(s)
    -f, --format <format>         Format for parameter values [default: table]  [possible values: table, csv, json,
                                  yaml]

ARGS:
    <KEY>    Name of parameter to show environment values

```