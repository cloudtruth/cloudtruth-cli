```console
$ cloudtruth actions imports task-steps --help
cloudtruth[EXE]-actions-imports-task-steps 
List task steps for the specified CloudTruth import

USAGE:
    cloudtruth[EXE] actions imports task-steps [FLAGS] [OPTIONS] <import-name>

FLAGS:
    -h, --help          Prints help information
        --show-times    Show create and modified times.
    -v, --values        Show import task step info values
    -V, --version       Prints version information

OPTIONS:
    -f, --format <format>       Import task step info format [default: table]  [possible values: table, csv, json, yaml]
    -i, --integration <name>    Integration name

ARGS:
    <import-name>    Import name

```