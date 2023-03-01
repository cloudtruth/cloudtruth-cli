```console
$ cloudtruth actions pushes list --help
cloudtruth[EXE]-actions-pushes-list 
List CloudTruth pushes

USAGE:
    cloudtruth[EXE] actions pushes list [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
        --show-times    Show create and modified times.
    -v, --values        Show push info values
    -V, --version       Prints version information

OPTIONS:
    -e, --env <env>             Filter by environment name
    -f, --format <format>       Push info output format [default: table]  [possible values: table, csv, json, yaml]
    -i, --integration <name>    Integration name
    -p, --project <project>     Filter by project name
    -t, --tag <tag>             Filter by tag name

```