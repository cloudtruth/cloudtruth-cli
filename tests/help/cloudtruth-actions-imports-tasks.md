```console
$ cloudtruth actions imports tasks --help
cloudtruth-actions-imports-tasks 
List tasks for the specified CloudTruth import

USAGE:
    cloudtruth actions imports tasks [FLAGS] [OPTIONS] <import-name>

FLAGS:
    -h, --help          Prints help information
        --show-times    Show create and modified times.
    -v, --values        Show import task info values
    -V, --version       Prints version information

OPTIONS:
    -f, --format <format>       Format for import task info [default: table]  [possible values: table, csv, json, yaml]
    -i, --integration <name>    Integration name

ARGS:
    <import-name>    Import name

```