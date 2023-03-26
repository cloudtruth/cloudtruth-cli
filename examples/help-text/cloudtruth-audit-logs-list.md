```console
$ cloudtruth audit-logs list --help
cloudtruth[EXE]-audit-logs-list 
List audit log details

USAGE:
    cloudtruth[EXE] audit-logs list [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --action <action>          Only show specified action [possible values: create, delete, update, nothing]
        --after <after>            Show audit log entries after specified date/time
        --before <before>          Show audit log entries before specified date/time
    -n, --name <contains>          Only show audit entries whose name contains the substring
        --env <environment>        Show audit log entries only from specified environment
    -f, --format <format>          Format for audit log details [default: table]  [possible values: table, csv, json,
                                   yaml]
    -m, --max <max-entries>        Limit the maximum number of entries, 0 for no limit. [default: 50]
    -t, --type <object-type>       Only show specified object type
        --parameter <parameter>    Show audit log entries only from specified parameter
        --project <project>        Show audit log entries only from specified project
    -u, --user <username>          Show audit log entries only from specified user

```