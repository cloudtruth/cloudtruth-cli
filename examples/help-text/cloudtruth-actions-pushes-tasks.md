```console
$ cloudtruth actions pushes tasks --help
cloudtruth[EXE]-actions-pushes-tasks 
List tasks for the specified CloudTruth push

USAGE:
    cloudtruth[EXE] actions pushes tasks [FLAGS] [OPTIONS] <push-name>

FLAGS:
    -h, --help          Prints help information
        --show-times    Show create and modified times.
    -v, --values        Show push task info values
    -V, --version       Prints version information

OPTIONS:
    -f, --format <format>       Push task info format [default: table]  [possible values: table, csv, json, yaml]
    -i, --integration <name>    Integration name

ARGS:
    <push-name>    Push name

```