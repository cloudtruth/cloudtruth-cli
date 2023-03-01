```console
$ cloudtruth actions imports set --help
cloudtruth-actions-imports-set 
Create/modify CloudTruth integration import

USAGE:
    cloudtruth actions imports set [FLAGS] [OPTIONS] <import-name>

FLAGS:
        --dry-run    Check that the import will work without doing it.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --desc <description>     Description for the import
    -i, --integration <name>     Integration name (required on create)
    -r, --rename <new-name>      New import name
        --region <region>        Region where import tasks run (create only) [default: us-east-1]
        --resource <resource>    Resource string (required for create, [default: '/{{ environment} }/{{ project }}/{{
                                 parameter }}'])
        --service <service>      Service for the import to use (create only) [default: ssm]  [possible values: ssm,
                                 secretsmanager]

ARGS:
    <import-name>    Import name

```