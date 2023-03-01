```console
$ cloudtruth actions pushes set --help
cloudtruth-actions-pushes-set 
Create/modify CloudTruth integration push

USAGE:
    cloudtruth actions pushes set [FLAGS] [OPTIONS] <push-name>

FLAGS:
        --no-force                 Make sure CloudTruth is the destination owner  [aliases: check-owner]
        --coerce-params            Include non-secret CloudTruth parameters, even in a secret store destination
        --dry-run                  Dry-run the push without changing any data
        --include-parameters       Include non-secret CloudTruth parameters in the values being pushed
        --include-secrets          Include secret CloudTruth parameters in the values being pushed
        --include-templates        Include templates in the values being pushed.
        --local                    Push only parameters defined in the selected project(s)
        --force                    Allow the push even if CloudTruth is not the destination owner [aliases: no-check-
                                   owner]
        --no-coerce-params         Do not include non-secret CloudTruth parameters in a secret store destination
        --no-dry-run               Create the push without being a dry-run
        --no-include-parameters    Do not include non-secret CloudTruth parameters in the values being pushed
        --no-include-secrets       Do not include secret CloudTruth parameters in the values being pushed
        --no-include-templates     Do not include templates in the values being pushed.
        --no-local                 Push all parameters (default)
    -h, --help                     Prints help information
    -V, --version                  Prints version information

OPTIONS:
    -d, --desc <description>             Description for the push
    -i, --integration <name>             Integration name (required for create)
    -r, --rename <new-name>              New push name
        --project <project>...           Project name(s) to be added
        --no-project <project>...        Project name(s) to be removed
        --region <region>                Region where push tasks run (create only) [default: us-east-1]
        --resource <resource>            Resource string (required for create, [default: '/{{ environment} }/{{ project
                                         }}/{{ parameter }}'])
        --service <service>              Service for the push to use (create only) [default: ssm]  [possible values:
                                         ssm, secretsmanager]
        --tag <environment:tag>...       Tag name(s) to be added
        --no-tag <environment:tag>...    Tag name(s) to be subtracted

ARGS:
    <push-name>    Push name

```