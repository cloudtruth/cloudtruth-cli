```console
$ cloudtruth users invitations set --help
cloudtruth[EXE]-users-invitations-set 
Create/update a Sigma Config user invitation

USAGE:
    cloudtruth[EXE] users invitations set [OPTIONS] <e-mail>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --role <role>    Role for invited user [default: viewer (on create)] [possible values: owner, admin, contrib,
                         viewer]

ARGS:
    <e-mail>    Email address for invitation

```