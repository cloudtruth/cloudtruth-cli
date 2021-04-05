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
                SubCommand::with_name("getit")
                    .about("Get implicit template for all parameters, \
                    limited to alphanumeric and underscore key names. Formats available are: \
                    dotenv, docker, and shell.")
                    .arg(Arg::with_name("contains")
                        .long("contains")
                        .help("Return parameters containing search")
                        .takes_value(true))
                    .arg(Arg::with_name("ends_with")
                        .long("ends-with")
                        .help("Return parameters ending with search")
                        .takes_value(true))
                    .arg(Arg::with_name("export")
                        .long("export")
                        .help("Add 'export' to each declaration"))
                    .arg(Arg::with_name("NAME")
                        .required(true)
                        .index(1))
                    .arg(Arg::with_name("secrets")
                        .long("secrets")
                        .help("Display the values of secret parameters"))
                    .arg(Arg::with_name("starts_with")
                        .long("starts-with")
                        .help("Return parameters starting with search")
                        .takes_value(true)),
                SubCommand::with_name("list")
                    .visible_alias("ls")
                    .about("List CloudTruth templates")
            ])
        )
        .subcommand(
            SubCommand::with_name("run")
                .visible_aliases(&["run", "r"])
                .about("Run a shell with the parameters in place")
                .args(&[
                    Arg::with_name("inheritance")
                        .long("inherit")
                        .short("i")
                        .takes_value(true)
                        .case_insensitive(true)
                        // TODO: Rick Porter 3/21 - pull subprocess::Inheritance enum value strings?
                        .possible_value("none")
                        .possible_value("underlay")
                        .possible_value( "overlay")
                        .possible_value("exclusive")
                        .default_value("overlay")
                        .help("Handle the relationship between local and CloudTruth environments"),
                    Arg::with_name("set")
                        .long("set")
                        .short("s")
                        .takes_value(true)
                        .multiple(true)
                        .help("Set the variables in this run, even possibly overriding the CloudTruth environment"),
                    Arg::with_name("remove")
                        .long("remove")
                        .short("r")
                        .takes_value(true)
                        .multiple(true)
                        .help("Remove the variables from the CloudTruth environment for this run"),
                    Arg::with_name("command")
                        .long("command")
                        .short("c")
                        .takes_value(true)
                        .help("Run this command"),
                    Arg::with_name("arguments")
                        .takes_value(true)
                        .multiple(true)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Treat the rest of the arguments as the command"),
                ])
        )
}
