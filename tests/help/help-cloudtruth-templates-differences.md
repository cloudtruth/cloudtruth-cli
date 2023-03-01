```console
$ cloudtruth templates differences --help
cloudtruth-templates-differences 
Show differences between templates

USAGE:
    cloudtruth templates differences [FLAGS] [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Compare unevaluated template bodies
    -s, --secrets    Compare evaluated secret values
    -V, --version    Prints version information

OPTIONS:
    -e, --env <ENV>...               Up to two environment(s) to be compared.
        --as-of <datetime|tag>...    Up to two times to be compared
    -c, --context <lines>            Number of lines of difference context [default: 3]

ARGS:
    <NAME>    Template name

```