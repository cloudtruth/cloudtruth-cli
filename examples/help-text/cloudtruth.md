```console
$ cloudtruth  --help
cloudtruth 1.2.8
Sigma Config <support@sigma-automate.com>
A command-line interface to the Sigma Config configuration management service.

USAGE:
    cloudtruth[EXE] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --api-key <api_key>    Sigma Config API key
    -e, --env <env>            The Sigma Config environment to work with
        --profile <profile>    The configuration profile from the application configuration file to use
        --project <project>    The Sigma Config project to work with
        --timeout <SECONDS>    Per-request timeout in seconds (overrides the profile/CLOUDTRUTH_REQUEST_TIMEOUT; default
                               90)

SUBCOMMANDS:
    actions            Manage Sigma Config actions [aliases: action, act, ac]
    audit-logs         Display audit logs [aliases: audit, aud, au, log, logs]
    backup             Manage backups of Sigma Config data [aliases: back, ba]
    completions        Generate shell completions for this application
    configuration      Configuration options for this application [aliases: config, conf, con, co, c]
    environments       Work with Sigma Config environments [aliases: environment, envs, env, e]
    generate           Generate items using Sigma Config service [aliases: gen, ge]
    groups             Manage Sigma Config user groups  [aliases: group, grp, gr, g]
    help               Prints this message or the help of the given subcommand(s)
    import             Perform imports into the Sigma Config environment [aliases: imp, im]
    integrations       Work with Sigma Config integrations [aliases: integration, integrate, integ, int, in]
    login              Sets up a Sigma Config configuration profile api_key
    logout             Removes a Sigma Config configuration profile api_key
    parameter-types    Manage parameter types in the Sigma Config environment [aliases: parameter-type, param-types,
                       param-type, types, type, ty]
    parameters         Work with Sigma Config parameters [aliases: parameter, params, param, par, pa, p]
    projects           Work with Sigma Config projects [aliases: project, proj]
    run                Run a shell with the parameters in place [aliases: run, ru, r]
    schema             View Sigma Config OpenAPI schema
    templates          Work with Sigma Config templates [aliases: template, temp, te, t]
    users              Work with Sigma Config users [aliases: user, us, u]
    versions           Manage Sigma Config CLI versions [aliases: version, vers, ver, ve, v]

```