```console
$ cloudtruth parameters drift --help
cloudtruth-parameters-drift 
Determine drift between current environment and project parameters

USAGE:
    cloudtruth parameters drift [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -s, --secrets    
    -v, --values     
    -V, --version    Prints version information

OPTIONS:
        --as-of <datetime|tag>    Date/time (or tag) of parameter value(s)
    -f, --format <format>         Format for differences [default: table]  [possible values: table, csv, json, yaml]

```