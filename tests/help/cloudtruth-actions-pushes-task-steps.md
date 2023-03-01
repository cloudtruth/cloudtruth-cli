```console
$ cloudtruth actions pushes task-steps --help
cloudtruth[EXE]-actions-pushes-task-steps 
List task steps for the specified CloudTruth push

USAGE:
    cloudtruth[EXE] actions pushes task-steps [FLAGS] [OPTIONS] <push-name>

FLAGS:
    -h, --help          Prints help information
        --show-times    Show create and modified times.
    -v, --values        Show push task step info values
    -V, --version       Prints version information

OPTIONS:
    -f, --format <format>       Push task steps info format [default: table]  [possible values: table, csv, json, yaml]
    -i, --integration <name>    Integration name

ARGS:
    <push-name>    Push name

```