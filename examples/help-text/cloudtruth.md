```console
$ cloudtruth  --help
cloudtruth 1.2.1
CloudTruth <support@cloudtruth.com>
A command-line interface to the CloudTruth configuration management service.

USAGE:
    cloudtruth[EXE] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --api-key <api_key>    CloudTruth API key
    -e, --env <env>            The CloudTruth environment to work with
        --profile <profile>    The configuration profile from the application configuration file to use
        --project <project>    The CloudTruth project to work with

SUBCOMMANDS:
    actions            Manage CloudTruth actions [aliases: action, act, ac]
    audit-logs         Display audit logs [aliases: audit, aud, au, log, logs]
    backup             Manage backups of CloudTruth data [aliases: back, ba]
    completions        Generate shell completions for this application
    configuration      Configuration options for this application [aliases: config, conf, con, co, c]
    environments       Work with CloudTruth environments [aliases: environment, envs, env, e]
    generate           Generate items using CloudTruth service [aliases: gen, ge]
    groups             Manage CloudTruth user groups  [aliases: group, grp, gr, g]
    help               Prints this message or the help of the given subcommand(s)
    import             Perform imports into the CloudTruth environment [aliases: imp, im]
    integrations       Work with CloudTruth integrations [aliases: integration, integrate, integ, int, in]
    login              Sets up a CloudTruth configuration profile api_key
    logout             Removes a CloudTruth configuration profile api_key
    parameter-types    Manage parameter types in the CloudTruth environment [aliases: parameter-type, param-types,
                       param-type, types, type, ty]
    parameters         Work with CloudTruth parameters [aliases: parameter, params, param, par, pa, p]
    projects           Work with CloudTruth projects [aliases: project, proj]
    run                Run a shell with the parameters in place [aliases: run, ru, r]
    schema             View CloudTruth OpenAPI schema
    templates          Work with CloudTruth templates [aliases: template, temp, te, t]
    users              Work with CloudTruth users [aliases: user, us, u]
    versions           Manage CloudTruth CLI versions [aliases: version, vers, ver, ve, v]

```