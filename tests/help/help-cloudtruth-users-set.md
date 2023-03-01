```console
$ cloudtruth users set --help
cloudtruth-users-set 
Create/update a CloudTruth service account

USAGE:
    cloudtruth users set [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --desc <description>    Account's description
        --role <role>           Account role [default: viewer (on create)] [possible values: owner, admin, contrib,
                                viewer]

ARGS:
    <NAME>    Account name

```