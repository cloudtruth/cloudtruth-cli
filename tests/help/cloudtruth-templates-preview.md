```console
$ cloudtruth templates preview --help
cloudtruth-templates-preview 
Evaluate the provided local template file without storing

USAGE:
    cloudtruth templates preview [FLAGS] [OPTIONS] <FILE>

FLAGS:
    -h, --help       Prints help information
    -s, --secrets    Display secret values in evaluation
    -V, --version    Prints version information

OPTIONS:
        --as-of <datetime|tag>    Date/time (or tag) of parameter value(s)

ARGS:
    <FILE>    File containing the template

```