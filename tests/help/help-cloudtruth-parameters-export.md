```console
$ cloudtruth parameters export --help
cloudtruth-parameters-export 
Export selected parameters to a known output format. Exported parameters are limited to alphanumeric and underscore  in
key names. Formats available are: dotenv, docker, and shell.

USAGE:
    cloudtruth parameters export [FLAGS] [OPTIONS] <FORMAT>

FLAGS:
        --export     Add 'export' to each declaration
    -h, --help       Prints help information
    -s, --secrets    Display the secret parameter values
    -V, --version    Prints version information

OPTIONS:
        --contains <contains>          Return parameters with keys containing search
        --as-of <datetime|tag>         Date/time (or tag) of parameter value(s)
        --ends-with <ends_with>        Return parameters with keys ending with search
        --starts-with <starts_with>    Return parameters starting with search

ARGS:
    <FORMAT>     [possible values: docker, dotenv, shell]

```