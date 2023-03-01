```console
$ cloudtruth actions pushes sync --help
cloudtruth-actions-pushes-sync 
Manually initiate action on existing push

USAGE:
    cloudtruth actions pushes sync [FLAGS] [OPTIONS] <push-name>

FLAGS:
        --no-force                 Make sure CloudTruth is the destination owner  [aliases: check-owner]
        --coerce-params            Include non-secret CloudTruth parameters, even in a secret store destination
        --dry-run                  Dry-run the push without changing any data
        --include-parameters       Include non-secret CloudTruth parameters in the values being pushed
        --include-secrets          Include secret CloudTruth parameters in the values being pushed
        --include-templates        Include templates in the values being pushed.
        --force                    Allow the push even if CloudTruth is not the destination owner [aliases: no-check-
                                   owner]
        --no-coerce-params         Do not include non-secret CloudTruth parameters in a secret store destination
        --no-dry-run               Create the push without being a dry-run
        --no-include-parameters    Do not include non-secret CloudTruth parameters in the values being pushed
        --no-include-secrets       Do not include secret CloudTruth parameters in the values being pushed
        --no-include-templates     Do not include templates in the values being pushed.
    -h, --help                     Prints help information
    -V, --version                  Prints version information

OPTIONS:
    -i, --integration <name>    Integration name

ARGS:
    <push-name>    Push name

```