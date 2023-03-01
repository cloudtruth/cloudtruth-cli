```console
$ cloudtruth parameters set --help
cloudtruth-parameters-set 
Set a value in the selected project/environment for an existing parameter or creates a new one if needed

USAGE:
    cloudtruth parameters set [FLAGS] [OPTIONS] <KEY>

FLAGS:
        --no-max          Remove the parameter rule maximum value
        --no-max-len      Remove the parameter rule maximum length value
        --no-min          Remove the parameter rule minimum value
        --no-min-len      Remove the parameter rule minimum length value
        --no-regex        Remove the parameter rule regex value
        --create-child    Create a parameter in the child project
        --generate        Generate a new value
    -h, --help            Prints help information
    -p, --prompt          Set the static value using unecho'd terminal
    -V, --version         Prints version information

OPTIONS:
    -f, --fqn <FQN>              Fully Qualified Name (FQN) reference for external parameter.
    -j, --jmes <jmes-path>       JMES path within FQN for external parameter
        --max <MAX>              Set parameter rule maximum value
        --max-len <MAX-LEN>      Set parameter rule maximum length value
        --min <MIN>              Set parameter rule minimum value
        --min-len <MIN-LEN>      Set parameter rule minimum length value
        --regex <REGEX>          Set parameter rule regex value
    -d, --desc <description>     Parameter description
    -e, --evaluate <evaluate>    Flags whether this value gets evaluated [possible values: true, false]
    -i, --input <input-file>     Read the static value from the local input file
    -r, --rename <new-name>      New parameter name
    -t, --type <param-type>      The parameter type. Fundamental types are: boolean, string, integer
        --secret <secret>        Flags whether this is a secret parameter [possible values: true, false]
    -v, --value <value>          Static parameter value

ARGS:
    <KEY>    Name of parameter to set

```