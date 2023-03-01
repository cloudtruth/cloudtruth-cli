```console
$ cloudtruth parameter-types set --help
cloudtruth[EXE]-parameter-types-set 
Set parameter type and rules

USAGE:
    cloudtruth[EXE] parameter-types set [FLAGS] [OPTIONS] <NAME>

FLAGS:
        --no-max        Remove the parameter rule maximum value
        --no-max-len    Remove the parameter rule maximum length value
        --no-min        Remove the parameter rule minimum value
        --no-min-len    Remove the parameter rule minimum length value
        --no-regex      Remove the parameter rule regex value
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
        --max <MAX>             Set parameter rule maximum value
        --max-len <MAX-LEN>     Set parameter rule maximum length value
        --min <MIN>             Set parameter rule minimum value
        --min-len <MIN-LEN>     Set parameter rule minimum length value
        --regex <REGEX>         Set parameter rule regex value
    -d, --desc <description>    Parameter type description
    -r, --rename <new-name>     New parameter type name
    -p, --parent <parent>       Parameter type parent

ARGS:
    <NAME>    Parameter type name

```