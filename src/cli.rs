use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, AppSettings,
    Arg, Shell, SubCommand,
};

pub fn binary_name() -> String {
    option_env!("CARGO_PKG_NAME")
        .unwrap_or("cloudtruth")
        .to_string()
}

pub fn build_cli() -> App<'static, 'static> {
    app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("api_key")
                .short("k")
                .long("api-key")
                .help("CloudTruth API key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("env")
                .help("The CloudTruth environment to work with")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("The profile from the application configuration file to use")
                .takes_value(true)
                .default_value("default")
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate shell completions for this application")
                .arg(Arg::with_name("SHELL").possible_values(&Shell::variants()).required(true))
        )
        .subcommand(SubCommand::with_name("config")
            .visible_alias("configuration")
            .about("Configuration options for this application")
            .subcommands(vec![
                SubCommand::with_name("edit")
                    .about("Edit your configuration data for this application")
                ])
        )
        .subcommand(
            SubCommand::with_name("environments")
                .visible_aliases(&["environment", "envs", "env", "e"])
                .about("Work with CloudTruth environments")
                .subcommands(vec![
                    SubCommand::with_name("list")
                        .visible_alias("ls")
                        .about("List CloudTruth environments")
                ])
        )
        .subcommand(
            SubCommand::with_name("parameters")
                .visible_aliases(&["parameter", "params", "param", "p"])
                .about("Work with CloudTruth parameters")
                .subcommands(vec![
                    SubCommand::with_name("get")
                        .about("Gets value for parameter in the selected environment")
                        .arg(Arg::with_name("KEY").required(true).index(1)),
                    SubCommand::with_name("list")
                        .visible_alias("ls")
                        .about("List CloudTruth parameters"),
                    SubCommand::with_name("set")
                        .about("Set a static value in the selected environment for an existing parameter or creates a new one if needed")
                        .arg(Arg::with_name("fully_qualified_name")
                            .long("fqn")
                            .help("A fully qualified name to a resource in a third party integration configured in CloudTruth")
                            .takes_value(true))
                        .arg(Arg::with_name("KEY")
                            .required(true)
                            .index(1)
                            .help("The name of the parameter"))
                        .arg(Arg::with_name("VALUE")
                            .required(true)
                            .index(2)
                            .help("If the FQN is not set, the value is simply the string value of the parameter, otherwise it is the JMESPath expression to apply against the resource identified by the FQN"))
                ]),
        )
        .subcommand(SubCommand::with_name("templates")
            .visible_aliases(&["template", "t"])
            .about("Work with CloudTruth templates")
            .subcommands(vec![
                SubCommand::with_name("get")
                    .about("Get an evaluated template from CloudTruth")
                    .arg(Arg::with_name("KEY").required(true).index(1)),
                SubCommand::with_name("list")
                    .visible_alias("ls")
                    .about("List CloudTruth templates"),
            ])
        )
}
