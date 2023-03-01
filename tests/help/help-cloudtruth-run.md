```console
$ cloudtruth run --help
cloudtruth-run 
Run a shell with the parameters in place

USAGE:
    cloudtruth run [FLAGS] [OPTIONS] [-- <arguments>...]

FLAGS:
    -h, --help          Prints help information
    -p, --permissive    Allow CloudTruth application variables through
        --strict        Fail when any parameters are unset
    -V, --version       Prints version information

OPTIONS:
    -c, --command <command>        Run this command
        --as-of <datetime|tag>     Date/time (or tag) of parameter value(s)
    -i, --inherit <inheritance>    Handle the relationship between local and CloudTruth environments [default: overlay]
                                   [possible values: none, underlay, overlay, exclusive]
    -r, --remove <remove>...       Remove the variables from the CloudTruth environment for this run
    -s, --set <set>...             Set the variables in this run, even possibly overriding the CloudTruth environment

ARGS:
    <arguments>...    Treat the rest of the arguments as the command

```