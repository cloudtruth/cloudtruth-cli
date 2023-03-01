```console
$ cloudtruth groups set --help
cloudtruth-groups-set 
Create/update a CloudTruth user group

USAGE:
    cloudtruth groups set [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --desc <description>                     Group description
    -r, --rename <new-name>                      Rename the group
        --add-user <username-to-add>...          Add user(s) to the group by name [aliases: add, user]
        --remove-user <username-to-remove>...    Remove user(s) from the group by name [aliases: remove, rm, rm-user]

ARGS:
    <NAME>    Group name

```