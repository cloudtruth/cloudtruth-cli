```console
$ cloudtruth environments tag set --help
cloudtruth[EXE]-environments-tag-set 
Create/update an environment tag

USAGE:
    cloudtruth[EXE] environments tag set [FLAGS] [OPTIONS] <env-name> <tag-name>

FLAGS:
    -c, --current    Update the tag's time to the current time
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --desc <description>    Tag's description
    -r, --rename <new-name>     New tag name
    -t, --time <timestamp>      Set the tag's timestamp value

ARGS:
    <env-name>    Environment name
    <tag-name>    Tag name

```