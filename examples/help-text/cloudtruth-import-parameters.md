```console
$ cloudtruth import parameters --help
cloudtruth[EXE]-import-parameters 
Import parameter values into a specified project and environment

USAGE:
    cloudtruth[EXE] import parameters [FLAGS] [OPTIONS] <project> <file>

FLAGS:
    -h, --help          Prints help information
    -n, --no-inherit    Do NOT inherit duplicate parameter values
        --preview       Simulate the import without saving any values
    -s, --secrets       Display secret values
        --show-times    Show import values created times
    -V, --version       Prints version information

OPTIONS:
    -e, --environment <environment>    Environment name into which parameters are imported
    -f, --format <format>              Format for imported parameter [default: table]  [possible values: table, csv,
                                       json, yaml]
    -i, --ignore <param-name>...       Parameters from the file to ignore
        --secret <param-name>...       Parameters from the file to treat as secrets

ARGS:
    <project>    Project name into which parameters are imported
    <file>       File that contains the text to import

```