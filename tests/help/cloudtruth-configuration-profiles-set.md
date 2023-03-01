```console
$ cloudtruth configuration profiles set --help
cloudtruth-configuration-profiles-set 
Create/modify CLI profile settings

USAGE:
    cloudtruth configuration profiles set [OPTIONS] <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --env <ENVIRONMENT>     Default environment for profile (use "" to remove)
    -p, --proj <PROJECT>        Default project for profile (use "" to remove)
    -s, --source <SOURCE>       Source (or parent) profile
    -k, --api-key <api_key>     CloudTruth API key
    -d, --desc <description>    Profile's description

ARGS:
    <NAME>    Profile name

```