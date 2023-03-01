```console
$ cloudtruth parameters differences --help
cloudtruth-parameters-differences 
Show differences between properties from two environments

USAGE:
    cloudtruth parameters differences [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                    Prints help information
    -i, --immediate_parameters    Show only immediate parameters (no inherited parameters)
    -s, --secrets                 Show secret values
    -V, --version                 Prints version information

OPTIONS:
    -e, --env <ENV>...                Up to two environment(s) to be compared.
        --as-of <datetime|tag>...     Up to two times to be compared
    -f, --format <format>             Display difference format [default: table]  [possible values: table, csv, json,
                                      yaml]
    -p, --property <properties>...    List of the properties to compare. [default: value]  [possible values: value,
                                      type, environment, fqn, jmes-path, raw, rule-count, secret, created-at, modified-
                                      at]

```