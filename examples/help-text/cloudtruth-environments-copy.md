```console
$ cloudtruth environments copy --help
cloudtruth[EXE]-environments-copy 
Copy an environment and its children to new environment(s)

USAGE:
    cloudtruth[EXE] environments copy [FLAGS] [OPTIONS] <src-name> <dest-name>

FLAGS:
    -h, --help         Prints help information
    -r, --recursive    Copy all descendants
    -V, --version      Prints version information

OPTIONS:
        --child-names <child-names-mapping>    Child names to copy as source=destination pairs separated by commas
                                               (Example: foo=bar,baz=qux). Requires --recursive option.
    -d, --desc <description>                   

ARGS:
    <src-name>     Source environment name for copy
    <dest-name>    Destination environment name for copy

```