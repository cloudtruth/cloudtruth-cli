```console
$ cloudtruth environments set --help
cloudtruth-environments-set 
Create/update a CloudTruth environment

USAGE:
    cloudtruth environments set [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --desc <description>    Environment's description
    -r, --rename <new-name>     New environment name
    -p, --parent <parent>       Environment's parent name (only used for create)

ARGS:
    <NAME>    Environment name

```