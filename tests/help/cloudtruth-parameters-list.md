```console
$ cloudtruth parameters list --help
cloudtruth[EXE]-parameters-list 
List CloudTruth parameters

USAGE:
    cloudtruth[EXE] parameters list [FLAGS] [OPTIONS]

FLAGS:
        --children                Display the parameters defined in a child project
        --evaluated               Display the evaluated values
        --external                Display the external values and FQN/JMES path.
    -h, --help                    Prints help information
    -i, --immediate_parameters    Show only immediate parameters (no inherited parameters)
        --parents                 Display the parameters defined in a parent project
        --rules                   Display the parameter rules.
    -s, --secrets                 Display the secret parameter values
        --show-times              Show create and modified times.
    -v, --values                  Display parameter information/values
    -V, --version                 Prints version information

OPTIONS:
        --as-of <datetime|tag>    Date/time (or tag) of parameter value(s)
    -f, --format <format>         Format for parameter values data [default: table]  [possible values: table, csv, json,
                                  yaml]

```