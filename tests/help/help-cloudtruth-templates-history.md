```console
$ cloudtruth templates history --help
cloudtruth-templates-history 
Display template history

USAGE:
    cloudtruth templates history [OPTIONS] [NAME]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --as-of <datetime|tag>    Date/time (or tag) of template history
    -f, --format <format>         Format for the template history [default: table]  [possible values: table, csv, json,
                                  yaml]

ARGS:
    <NAME>    Template name (optional)

```