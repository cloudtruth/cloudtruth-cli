```console
$ cloudtruth templates get --help
cloudtruth[EXE]-templates-get 
Get an evaluated template from CloudTruth

USAGE:
    cloudtruth[EXE] templates get [FLAGS] [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Display unevaluated template body
    -s, --secrets    Display secret values in evaluation
    -V, --version    Prints version information

OPTIONS:
        --as-of <datetime|tag>     Date/time (or tag) of template (and parameters)

ARGS:
    <NAME>    Template name

```