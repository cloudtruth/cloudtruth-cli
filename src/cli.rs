use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, AppSettings,
    Arg, Shell, SubCommand,
};

pub const CONFIRM_FLAG: &str = "confirm";
pub const DESCRIPTION_OPT: &str = "description";
pub const FORMAT_OPT: &str = "format";
pub const NAME_ARG: &str = "NAME";
pub const RENAME_OPT: &str = "rename";
pub const SECRETS_FLAG: &str = "secrets";
pub const TEMPLATE_FILE_OPT: &str = "FILE";
pub const VALUES_FLAG: &str = "values";
pub const KEY_ARG: &str = "KEY";

pub const DELETE_SUBCMD: &str = "delete";
pub const EDIT_SUBCMD: &str = "edit";
pub const GET_SUBCMD: &str = "get";
pub const LIST_SUBCMD: &str = "list";
pub const SET_SUBCMD: &str = "set";

const DELETE_ALIASES: &[&str] = &["del", "d"];
const LIST_ALIASES: &[&str] = &["ls", "l"];

pub fn binary_name() -> String {
    option_env!("CARGO_PKG_NAME")
        .unwrap_or("cloudtruth")
        .to_string()
}

fn table_format_options() -> Arg<'static, 'static> {
    Arg::with_name(FORMAT_OPT)
        .short("f")
        .long(FORMAT_OPT)
        .takes_value(true)
        .default_value("table")
        .possible_values(&["table", "csv", "json", "yaml"])
}

fn values_flag() -> Arg<'static, 'static> {
    Arg::with_name(VALUES_FLAG).short("v").long(VALUES_FLAG)
}

fn secrets_display_flag() -> Arg<'static, 'static> {
    Arg::with_name(SECRETS_FLAG).short("s").long(SECRETS_FLAG)
}

fn confirm_flag() -> Arg<'static, 'static> {
    Arg::with_name(CONFIRM_FLAG)
        .alias(CONFIRM_FLAG)
        .short("y")
        .long("yes")
        .help("Avoid confirmation prompt(s)")
}

fn rename_option() -> Arg<'static, 'static> {
    Arg::with_name(RENAME_OPT)
        .short("r")
        .long(RENAME_OPT)
        .takes_value(true)
}

fn description_option() -> Arg<'static, 'static> {
    Arg::with_name(DESCRIPTION_OPT)
        .short("d")
        .long("desc")
        .takes_value(true)
}

fn template_body() -> Arg<'static, 'static> {
    Arg::with_name(TEMPLATE_FILE_OPT).help("File containing the template")
}

fn name_arg() -> Arg<'static, 'static> {
    Arg::with_name(NAME_ARG).required(true).index(1)
}

fn key_arg() -> Arg<'static, 'static> {
    Arg::with_name(KEY_ARG).required(true).index(1)
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
        .subcommand(SubCommand::with_name("configuration")
            .visible_aliases(&["config", "conf", "con"])
            .about("Configuration options for this application")
            .subcommands(vec![
                SubCommand::with_name(EDIT_SUBCMD)
                    .about("Edit your configuration data for this application"),
                SubCommand::with_name(LIST_SUBCMD)
                    .visible_aliases(LIST_ALIASES)
                    .arg(values_flag().help("Display profile information/values"))
                    .arg(table_format_options().help("Display profile value info format"))
                    .arg(secrets_display_flag().help("Display API key values"))
                    .about("List CloudTruth profiles in the local config file"),
                SubCommand::with_name("current")
                    .visible_aliases(&["curr", "cur"])
                    .arg(table_format_options().help("Display table format"))
                    .arg(secrets_display_flag().help("Display API key values"))
                    .arg( Arg::with_name("extended")
                        .hidden(true)
                        .short("x").
                        help("Show extended values"))
                    .about("Show the current arguments and their sources.")
                ])
        )
        .subcommand(
            SubCommand::with_name("environments")
                .visible_aliases(&["environment", "envs", "env", "e"])
                .about("Work with CloudTruth environments")
                .subcommands(vec![
                    SubCommand::with_name(DELETE_SUBCMD)
                        .visible_aliases(DELETE_ALIASES)
                        .about("Delete specified CloudTruth environment")
                        .arg(name_arg().help("Environment name"))
                        .arg(confirm_flag()),
                    SubCommand::with_name(LIST_SUBCMD)
                        .visible_aliases(LIST_ALIASES)
                        .about("List CloudTruth environments")
                        .arg(values_flag().help("Display environment information/values"))
                        .arg(table_format_options().help("Format for environment values data")),
                    SubCommand::with_name(SET_SUBCMD)
                        .about("Create/update a CloudTruth environment")
                        .arg(name_arg().help("Environment name"))
                        .arg(description_option().help("Environment's description"))
                        .arg(rename_option().help("New environment name"))
                        .arg(Arg::with_name("parent")
                            .short("p")
                            .long("parent")
                            .takes_value(true)
                            .help("Environment's parent name (only used for create)")),
                    SubCommand::with_name("tree")
                        .visible_aliases(&["tr", "t"])
                        .about("Show a tree representation of the environments")
                        .arg(name_arg()
                            .help("Show this environment with children")
                            .required(false)
                            .default_value("default")),
                ])
        )
        .subcommand(SubCommand::with_name("login")
            .about("Login into the CloudTruth server")
            .arg(confirm_flag()))
        .subcommand(SubCommand::with_name("logout")
            .about("Logout of the CloudTruth server")
            .arg(confirm_flag()))
        .subcommand(
            SubCommand::with_name("integrations")
                .visible_aliases(&["integration", "integrate", "integ", "int"])
                .about("Work with CloudTruth integrations")
                .subcommands(vec![
                    SubCommand::with_name("explore")
                        .visible_aliases(&["exp", "ex", "e"])
                        .about("Explore integrations by Fully Qualified Name (FQN).")
                        .arg(Arg::with_name("FQN")
                            .index(1)
                            .takes_value(true)
                            .help("Integration FQN"))
                        .arg(table_format_options().help("Format integration values data."))
                        .arg(values_flag().help("Display integration values")),
                    SubCommand::with_name(LIST_SUBCMD)
                        .visible_aliases(LIST_ALIASES)
                        .about("List CloudTruth integrations")
                        .arg(values_flag().help("Display integration information/values"))
                        .arg(table_format_options().help("Format for integration values data")),
                ])
        )
        .subcommand(
            SubCommand::with_name("parameters")
                .visible_aliases(&["parameter", "params", "param", "p"])
                .about("Work with CloudTruth parameters")
                .subcommands(vec![
                    SubCommand::with_name(DELETE_SUBCMD)
                        .visible_aliases(DELETE_ALIASES)
                        .about("Delete the parameter from the project")
                        .arg(confirm_flag())
                        .arg(key_arg().help("Name of parameter to delete")),
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
                    SubCommand::with_name("environment")
                        .visible_aliases(&["environ", "env"])
                        .about("Shows values across environments")
                        .arg(key_arg().help("Name of parameter to show environment values"))
                        .arg(Arg::with_name("all").short("a").long("all").help("Show even unset environments."))
                        .arg(table_format_options().help("Format for parameter values"))
                        .arg(secrets_display_flag().help("Display secret values in environments")),
                    SubCommand::with_name(GET_SUBCMD)
                        .about("Gets value for parameter in the selected environment")
                        .arg(key_arg().help("Name of parameter to get")),
                    SubCommand::with_name(LIST_SUBCMD)
                        .visible_aliases(LIST_ALIASES)
                        .about("List CloudTruth parameters")
                        .arg(Arg::with_name("dynamic")
                            .long("dynamic")
                            .help("Display the dynamic values and FQN/JMES path."))
                        .arg(values_flag().help("Display parameter information/values"))
                        .arg(table_format_options().help("Format for parameter values data"))
                        .arg(secrets_display_flag().help("Display the secret parameter values")),
                    SubCommand::with_name(SET_SUBCMD)
                        .about(concat!("Set a value in the selected project/environment for ",
                            "an existing parameter or creates a new one if needed"))
                        .arg(key_arg().help("Name of parameter to set"))
                        .arg(description_option().help("Parameter description"))
                        .arg(Arg::with_name("FQN")
                            .short("f")
                            .long("fqn")
                            .takes_value(true)
                            .help("Fully Qualified Name (FQN) reference for dynamic parameter."))
                        .arg(Arg::with_name("input-file")
                            .short("i")
                            .long("input")
                            .takes_value(true)
                            .help("Read the static value from the local input file"))
                        .arg(Arg::with_name("JMES")
                            .short("j")
                            .long("jmes")
                            .takes_value(true)
                            .help("JMES path within FQN for dynamic parameter"))
                        .arg(Arg::with_name("prompt")
                            .short("p")
                            .long("prompt")
                            .help("Set the static value using unecho'd terminal"))
                        .arg(rename_option().help("New parameter name"))
                        .arg(Arg::with_name("secret")
                            .long("secret")
                            .takes_value(true)
                            .possible_values(&["true", "false"])
                            .help("Flags whether this is a secret parameter"))
                        .arg(Arg::with_name("value")
                            .short("v")
                            .long("value")
                            .takes_value(true)
                            .help("Static parameter value")),
                    SubCommand::with_name("unset")
                        .about(concat!("Remove a value/override from the selected ",
                            "project/environment and leaves the parameter in place."))
                        .arg(key_arg().help("Name of parameter to unset")),
                    SubCommand::with_name("differences")
                        .visible_aliases(&["difference", "differ", "diff"])
                        .about("Show differences between properties from two environments")
                        .arg(Arg::with_name("ENV1")
                            .required(true)
                            .index(1)
                            .help("Name of first environment for comparison."))
                        .arg(Arg::with_name("ENV2")
                            .required(true)
                            .index(2)
                            .help("Name of second environment for comparison."))
                        .arg(Arg::with_name("properties")
                            .short("p")
                            .long("property")
                            .possible_values(&["value", "environment", "fqn", "jmes-path", "secret"])
                            .multiple(true)
                            .default_value("value")
                            .help("List of the properties to compare."))
                        .arg(table_format_options().help("Display difference format"))
                        .arg(secrets_display_flag().help("Show secret values")),
                ]),
        )
        .subcommand(SubCommand::with_name("templates")
            .visible_aliases(&["template", "temp", "t"])
            .about("Work with CloudTruth templates")
            .subcommands(vec![
                SubCommand::with_name(DELETE_SUBCMD)
                    .visible_aliases(DELETE_ALIASES)
                    .about("Delete the CloudTruth template")
                    .arg(confirm_flag())
                    .arg(name_arg().help("Template name")),
                SubCommand::with_name(EDIT_SUBCMD)
                    .about("Edit the specified template")
                    .arg(name_arg().help("Template name")),
                SubCommand::with_name(GET_SUBCMD)
                    .about("Get an evaluated template from CloudTruth")
                    .arg(Arg::with_name("raw")
                        .short("r")
                        .long("raw")
                        .help("Get the raw, unevaluated template text"))
                    .arg(secrets_display_flag().help("Display secret values in evaluation"))
                    .arg(name_arg().help("Template name")),
                SubCommand::with_name(LIST_SUBCMD)
                    .visible_aliases(LIST_ALIASES)
                    .about("List CloudTruth templates")
                    .arg(values_flag().help("Display template information/values"))
                    .arg(table_format_options().help("Format for template values data")),
                SubCommand::with_name("preview")
                    .about("Evaluate the provided template without storing")
                    .visible_aliases(&["prev", "pre"])
                    .arg(template_body().required(true).index(1))
                    .arg(secrets_display_flag().help("Display secret values in evaluation")),
                SubCommand::with_name(SET_SUBCMD)
                    .arg(name_arg().help("Template name"))
                    .arg(template_body().takes_value(true).short("b").long("body"))
                    .arg(rename_option().help("New template name"))
                    .arg(description_option().help("Template description"))
                    .about("Set the CloudTruth template"),
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
                    SubCommand::with_name(DELETE_SUBCMD)
                        .visible_aliases(DELETE_ALIASES)
                        .about("Delete specified CloudTruth project")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Project name"))
                        .arg(confirm_flag()),
                    SubCommand::with_name(LIST_SUBCMD)
                        .visible_aliases(LIST_ALIASES)
                        .about("List CloudTruth projects")
                        .arg(values_flag().help("Display project information/values"))
                        .arg(table_format_options().help("Format for project values data")),
                    SubCommand::with_name(SET_SUBCMD)
                        .about("Create/update a CloudTruth project")
                        .arg(Arg::with_name("NAME")
                            .index(1)
                            .required(true)
                            .help("Project name"))
                        .arg(rename_option().help("New project name"))
                        .arg(description_option().help("Project's description")),
                ])
        )
}
