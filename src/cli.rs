use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, AppSettings,
    Arg, Shell, SubCommand,
};

pub fn binary_name() -> String {
    option_env!("CARGO_PKG_NAME")
        .unwrap_or("cloudtruth")
        .to_string()
}

fn table_format_options() -> Arg<'static, 'static> {
    Arg::with_name("format")
        .short("f")
        .long("format")
        .takes_value(true)
        .default_value("table")
        .possible_values(&["table", "csv"])
}

fn values_flag() -> Arg<'static, 'static> {
    Arg::with_name("values").short("v").long("values")
}

fn secrets_display_flag() -> Arg<'static, 'static> {
    Arg::with_name("secrets").short("s").long("secrets")
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
                .long("profile")
                .help("The configuration profile from the application configuration file to use")
                .takes_value(true)
                .default_value("default")
        )
        .arg(
            Arg::with_name("project")
                .long("project")
                .help("The CloudTruth project to work with")
                .takes_value(true)
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
                    .about("Edit your configuration data for this application"),
                SubCommand::with_name("list")
                    .visible_alias("ls")
                    .arg(values_flag().help("Display profile information/values"))
                    .arg(table_format_options().help("Display profile value info format"))
                    .arg(secrets_display_flag().help("Display API key values"))
                    .about("List CloudTruth profiles in the local config file"),
                ])
        )
        .subcommand(
            SubCommand::with_name("environments")
                .visible_aliases(&["environment", "envs", "env", "e"])
                .about("Work with CloudTruth environments")
                .subcommands(vec![
                    SubCommand::with_name("delete")
                        .visible_alias("del")
                        .about("Delete specified CloudTruth environment")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Environment name"))
                        .arg(Arg::with_name("confirm")
                            .long("confirm")
                            .help("Avoid confirmation prompt")),
                    SubCommand::with_name("list")
                        .visible_alias("ls")
                        .about("List CloudTruth environments")
                        .arg(values_flag().help("Display environment information/values"))
                        .arg(table_format_options().help("Format for environment values data")),
                    SubCommand::with_name("set")
                        .about("Create/update a CloudTruth environment")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Environment name"))
                        .arg(Arg::with_name("description")
                            .short("d")
                            .long("desc")
                            .takes_value(true)
                            .help("Environment's description"))
                        .arg(Arg::with_name("parent")
                            .short("p")
                            .long("parent")
                            .takes_value(true)
                            .help("Environment's parent name (only used for create)")),
                ])
        )
        .subcommand(
            SubCommand::with_name("parameters")
                .visible_aliases(&["parameter", "params", "param", "p"])
                .about("Work with CloudTruth parameters")
                .subcommands(vec![
                    SubCommand::with_name("delete")
                        .visible_alias("del")
                        .about("Delete the parameter from the project")
                        .arg(Arg::with_name("KEY").required(true)),
                    SubCommand::with_name("export")
                        .about(concat!("Export selected parameters to a known output format. ",
                            "Exported parameters are limited to alphanumeric and underscore  in ",
                            "key names. Formats available are: dotenv, docker, and shell."))
                        .arg(Arg::with_name("contains")
                            .long("contains")
                            .help("Return parameters with keys containing search")
                            .takes_value(true))
                        .arg(Arg::with_name("ends_with")
                            .long("ends-with")
                            .help("Return parameters with keys ending with search")
                            .takes_value(true))
                        .arg(Arg::with_name("export")
                            .long("export")
                            .help("Add 'export' to each declaration"))
                        .arg(Arg::with_name("FORMAT")
                            .required(true)
                            .possible_value("docker")
                            .possible_value("dotenv")
                            .possible_value("shell")
                            .index(1))
                        .arg(secrets_display_flag().help("Display the secret parameter values"))
                        .arg(Arg::with_name("starts_with")
                            .long("starts-with")
                            .help("Return parameters starting with search")
                            .takes_value(true)),
                    SubCommand::with_name("get")
                        .about("Gets value for parameter in the selected environment")
                        .arg(Arg::with_name("KEY").required(true).index(1)),
                    SubCommand::with_name("list")
                        .visible_alias("ls")
                        .about("List CloudTruth parameters")
                        .arg(values_flag().help("Display parameter information/values"))
                        .arg(table_format_options().help("Format for parameter values data"))
                        .arg(secrets_display_flag().help("Display the secret parameter values")),
                    SubCommand::with_name("set")
                        .about(concat!("Set a static value in the selected project/environment for ",
                            "an existing parameter or creates a new one if needed"))
                        .arg(Arg::with_name("KEY").required(true).index(1))
                        .arg(Arg::with_name("description")
                            .takes_value(true)
                            .short("d")
                            .long("desc")
                            .help("Parameter description"))
                        .arg(Arg::with_name("input-file")
                            .short("i")
                            .long("input")
                            .takes_value(true)
                            .help("Read the value from the input file"))
                        .arg(Arg::with_name("prompt")
                            .short("p")
                            .long("prompt")
                            .help("Set the value using unecho'd terminal"))
                        .arg(Arg::with_name("secret")
                            .long("secret")
                            .takes_value(true)
                            .possible_values(&["true", "false"])
                            .help("Flags whether this is a secret parameter"))
                        .arg(Arg::with_name("value")
                            .short("v")
                            .long("value")
                            .takes_value(true)
                            .help("Parameter value")),
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
                    .about("List CloudTruth templates")
                    .arg(values_flag().help("Display template information/values"))
                    .arg(table_format_options().help("Format for template values data"))
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
                    Arg::with_name("permissive")
                        .long("permissive")
                        .short("p")
                        .help("Allow CloudTruth application variables through")
                ])
        )
        .subcommand(
            SubCommand::with_name("projects")
                .visible_aliases(&["project", "proj"])
                .about("Work with CloudTruth projects")
                .subcommands(vec![
                    SubCommand::with_name("delete")
                        .visible_alias("del")
                        .about("Delete specified CloudTruth project")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Project name"))
                        .arg(Arg::with_name("confirm")
                            .long("confirm")
                            .help("Avoid confirmation prompt")),
                    SubCommand::with_name("list")
                        .visible_alias("ls")
                        .about("List CloudTruth projects")
                        .arg(values_flag().help("Display project information/values"))
                        .arg(table_format_options().help("Format for project values data")),
                    SubCommand::with_name("set")
                        .about("Create/update a CloudTruth project")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Project name"))
                        .arg(Arg::with_name("description")
                            .short("d")
                            .long("desc")
                            .takes_value(true)
                            .help("Project's description")),
                ])
        )
}
