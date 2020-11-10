use clap::{App, AppSettings, Arg, Shell, SubCommand};

pub fn binary_name() -> String {
    option_env!("CARGO_PKG_NAME")
        .unwrap_or("cloudtruth")
        .to_string()
}

pub fn build_cli() -> App<'static, 'static> {
    App::new("CloudTruth")
        .version("0.1.0")
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
                        .arg(Arg::with_name("KEY").required(true).index(1))
                        .arg(Arg::with_name("VALUE").required(true).index(2))
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
