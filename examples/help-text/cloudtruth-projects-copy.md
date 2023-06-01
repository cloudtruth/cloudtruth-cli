```console
$ cloudtruth projects copy --help
cloudtruth[EXE]-projects-copy 
Copy a project and its children to new project(s)

USAGE:
    cloudtruth[EXE] projects copy [FLAGS] [OPTIONS] <src-name> <dest-name>

FLAGS:
    -h, --help         Prints help information
    -r, --recursive    Copy all descendants
    -V, --version      Prints version information

OPTIONS:
        --child-names <child-names-mapping>    Child names to copy as source=destination pairs separated by commas
                                               (Example: foo=bar,baz=qux). Requires --recursive option.
    -d, --desc <description>                   

ARGS:
    <src-name>     Source project name to copy
    <dest-name>    Destination project name

```