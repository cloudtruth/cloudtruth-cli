extern crate rpassword;

mod actions;
mod audit_logs;
mod cli;
mod config;
mod configuration;
mod database;
mod environments;
mod integrations;
mod lib;
mod login;
mod logout;
mod parameters;
mod projects;
mod run;
mod schema;
mod subprocess;
mod table;
mod templates;
mod users;

use crate::actions::process_actions_command;
use crate::audit_logs::process_audit_log_command;
use crate::config::env::ConfigEnv;
use crate::config::{Config, CT_PROFILE, DEFAULT_ENV_NAME};
use crate::configuration::process_config_command;
use crate::database::{Environments, OpenApiConfig, Projects};
use crate::environments::process_environment_command;
use crate::integrations::process_integrations_command;
use crate::login::process_login_command;
use crate::logout::process_logout_command;
use crate::parameters::process_parameters_command;
use crate::projects::process_project_command;
use crate::run::process_run_command;
use crate::schema::process_schema_command;
use crate::templates::process_templates_command;
use crate::users::process_users_command;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use color_eyre::Report;
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::io;
use std::io::{stdin, stdout, Write};
use std::process;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// The `DEL_CONFIRM` is the default value for delete confirmation across different types
const DEL_CONFIRM: Option<bool> = Some(false);
const REDACTED: &str = "*****";
const FILE_READ_ERR: &str = "Failed to read value from file.";
const ISO8601: &str = "%Y-%m-%dT%H:%M:%S%.fZ";
pub const SEPARATOR: &str = "=========================";
pub const API_KEY_PAGE: &str = "\"API Access\"";

#[derive(Clone, Debug)]
pub enum ApplicationError {
    InvalidApiUrl(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::InvalidApiUrl(api_url) => {
                write!(f, "No equivalent application URL for API: {}", api_url)
            }
        }
    }
}

impl error::Error for ApplicationError {}

pub struct ResolvedIds {
    pub env_name: Option<String>,
    pub env_id: Option<String>,
    pub proj_name: Option<String>,
    pub proj_id: Option<String>,
}

impl ResolvedIds {
    fn environment_display_name(&self) -> String {
        self.env_name
            .clone()
            .unwrap_or_else(|| DEFAULT_ENV_NAME.to_string())
    }

    fn project_display_name(&self) -> String {
        self.proj_name.clone().unwrap_or_default()
    }

    fn project_id(&self) -> &str {
        self.proj_id.as_deref().unwrap()
    }

    fn environment_id(&self) -> &str {
        self.env_id.as_deref().unwrap()
    }
}

/// Print a message to stderr in the specified color.
fn stderr_message(message: String, color: Color) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(color));

    stderr.set_color(&color_spec).unwrap_or_default();
    writeln!(&mut stderr, "{}", message).unwrap_or_default();
    stderr.reset().unwrap_or_default();
}

/// Print the provided message to stderr in 'Yellow'.
fn warning_message(message: String) {
    stderr_message(message, Color::Yellow);
}

/// Print the provided message to stderr in 'Red'.
fn error_message(message: String) {
    stderr_message(message, Color::Red);
}

/// Print the provided message to stderr in 'Cyan'.
fn help_message(message: String) {
    stderr_message(message, Color::Cyan);
}

fn error_no_environment_message(env_name: &str) {
    error_message(format!(
        "The '{}' environment could not be found in your account.",
        env_name,
    ));
}

/// Insures the configuration is valid.
///
/// If there are errors, it will print the error/help and exit.
/// If only warnings happen, it will print the warning and keep going.
fn check_config() -> Result<()> {
    if let Some(issues) = Config::global().validate() {
        // print the warnings first, so the user sees them (even when errors are present)
        let warnings = issues.warnings;
        if !warnings.is_empty() {
            for message in warnings {
                warning_message(message);
            }
        }

        let errors = issues.errors;
        if !errors.is_empty() {
            for err in errors {
                error_message(err.message);
                help_message(err.help_message);
            }
            process::exit(1)
        }
    }
    Ok(())
}

/// Add "WARN:" prefix to the message, and print it to stderr
fn warn_user(message: String) {
    warning_message(format!("WARN: {}", message));
}

/// Simple method for standardizing the message when no sub-command is executed.
fn warn_missing_subcommand(command: &str) {
    warn_user(format!("No '{}' sub-command executed.", command));
}

/// Method for standardizing message about list of warnings.
fn warn_unresolved_params(errors: &[String]) {
    if !errors.is_empty() {
        warning_message(format!(
            "Errors resolving parameters:\n{}\n",
            errors.join("\n")
        ));
    }
}

/// Format the strings in the list of errors
fn format_param_error(param_name: &str, param_err: &str) -> String {
    format!("   {}: {}", param_name, param_err)
}

/// Prompts the user for 'y/n' output.
///
/// If the user answers 'y' (case insensitive), 'true' is returned.
/// If the user answers 'n' (case insensitive), 'false' is returned.
/// The prompt will be repeated upto 3 times if the users does not enter 'y|n'. If the
/// max tries are exceeded, it returns 'false'.
fn user_confirm(message: String, default: Option<bool>) -> bool {
    let max_tries = 3;
    let mut confirmed = false;
    let action = match default {
        None => "y/n",
        Some(true) => "Y/n",
        Some(false) => "y/N",
    };

    for _ in 0..max_tries {
        let mut input = String::new();
        print!("{}? ({}) ", message, action);
        stdout().flush().unwrap();
        let _ = stdin().read_line(&mut input);
        input = input.trim().to_string().to_lowercase();
        if input.is_empty() {
            if let Some(value) = default {
                confirmed = value;
                break;
            }
        }
        if input.as_str() == "y" || input.as_str() == "yes" {
            confirmed = true;
            break;
        }
        if input.as_str() == "n" || input.as_str() == "no" {
            break;
        }
    }
    confirmed
}

/// Resolves the environment and project strings.
///
/// If either fails, it prints an error and exits.
/// On success, it returns a `ResolvedIds` structure that contains ids to avoid needing to resolve
/// the names again.
fn resolve_ids(config: &Config, rest_cfg: &OpenApiConfig) -> Result<ResolvedIds> {
    // The `err` value is used to allow accumulation of multiple errors to the user.
    let mut err = false;
    let env = config.environment.as_deref().unwrap_or(DEFAULT_ENV_NAME);
    let proj = config.project.as_deref();
    let environments = Environments::new();
    let env_id = environments.get_id(rest_cfg, env)?;
    if env_id.is_none() {
        error_no_environment_message(env);
        err = true;
    }

    let mut proj_id = None;
    if let Some(proj_str) = proj {
        let projects = Projects::new();
        proj_id = projects.get_id(rest_cfg, proj_str)?;
        if proj_id.is_none() {
            error_message(format!(
                "The '{}' project could not be found in your account.",
                proj_str,
            ));
            err = true;
        }
    } else {
        error_message("No project name was provided!".to_owned());
        err = true;
    }

    // if any errors were encountered, exit with an error code
    if err {
        process::exit(2);
    }

    Ok(ResolvedIds {
        env_name: Some(env.to_string()),
        env_id,
        proj_name: proj.map(String::from),
        proj_id,
    })
}

/// Get the web application URL for the `API_KEY_PAGE`
fn get_api_access_url(api_url: &str) -> Result<String> {
    // remove the any trailing '/'
    let mut api = api_url.to_string();
    if api.ends_with('/') {
        api.truncate(api.len() - 1);
    }
    let api_access_path = "organization/api";
    if api.starts_with("https://localhost:8000") {
        return Ok(format!("https://localhost:7000/{}", api_access_path));
    }
    if api.starts_with("https://api.") && api.ends_with("cloudtruth.io") {
        return Ok(format!(
            "{}/{}",
            api.replace("https://api", "https://app"),
            api_access_path
        ));
    }
    Err(Report::new(ApplicationError::InvalidApiUrl(
        api_url.to_string(),
    )))
}

/// Process the 'completion' sub-command
fn process_completion_command(subcmd_args: &ArgMatches) {
    let shell = subcmd_args.value_of("SHELL").unwrap();

    cli::build_cli().gen_completions_to(
        cli::binary_name(),
        shell.parse().unwrap(),
        &mut io::stdout(),
    );
}

/// Quick pass at providing a current-time in an acceptable time format for the server.
fn current_time() -> String {
    let now = Utc::now();
    now.format(ISO8601).to_string()
}

/// Takes an optional CLI argument (`Option<&str>`) attempts to parse it to a valid `DateTime`, and
/// returns the ISO format that the API expects.
///
/// If this is not a recognized date-time format, it will return `None`.
fn parse_datetime(input: Option<&str>) -> Option<String> {
    if let Some(orig) = input {
        if let Ok(rfc2822) = DateTime::parse_from_rfc2822(orig) {
            Some(rfc2822.format(ISO8601).to_string())
        } else if let Ok(rfc3339) = DateTime::parse_from_rfc3339(orig) {
            Some(rfc3339.format(ISO8601).to_string())
        } else if let Ok(datetime) = NaiveDateTime::parse_from_str(orig, ISO8601) {
            Some(datetime.format(ISO8601).to_string())
        } else if let Ok(datetime) = NaiveDateTime::parse_from_str(orig, "%Y-%m-%dT%H:%M:%S%.f") {
            Some(datetime.format(ISO8601).to_string())
        } else if let Ok(time_only) = NaiveTime::parse_from_str(orig, "%H:%M:%S%.f") {
            let now = Utc::now();
            let new_str = format!(
                "{}-{}-{}T{}Z",
                now.year(),
                now.month(),
                now.day(),
                time_only.to_string()
            );
            let dt = NaiveDateTime::parse_from_str(&new_str, ISO8601).unwrap();
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(full_date) = NaiveDate::parse_from_str(orig, "%Y-%m-%d") {
            let new_str = format!("{}T00:00:00Z", full_date.to_string());
            let dt = NaiveDateTime::parse_from_str(&new_str, ISO8601).unwrap();
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(us_date) = NaiveDate::parse_from_str(orig, "%m-%d-%Y") {
            let new_str = format!("{}T00:00:00Z", us_date.to_string());
            let dt = NaiveDateTime::parse_from_str(&new_str, ISO8601).unwrap();
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(us_date) = NaiveDate::parse_from_str(orig, "%m/%d/%Y") {
            let new_str = format!("{}T00:00:00Z", us_date.to_string());
            let dt = NaiveDateTime::parse_from_str(&new_str, ISO8601).unwrap();
            Some(dt.format(ISO8601).to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Returns a tag value, if the input value is not a recognized date-time format.
fn parse_tag(input: Option<&str>) -> Option<String> {
    if parse_datetime(input).is_some() {
        None
    } else {
        input.map(String::from)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let profile_env = ConfigEnv::get_override(CT_PROFILE);
    let matches = cli::build_cli().get_matches();

    let api_key = matches.value_of("api_key");
    let profile_arg = matches.value_of("profile");
    let profile_name = matches
        .value_of("profile")
        .or_else(|| profile_env.as_deref());
    let env_name = matches.value_of("env");
    let proj_name = matches.value_of("project");

    if let Some(matches) = matches.subcommand_matches("completions") {
        process_completion_command(matches);
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("configuration") {
        process_config_command(matches, profile_arg, api_key, proj_name, env_name)?;
        process::exit(0)
    }

    // wait until after processing the config command to load the config -- if we fail to load the
    // config, we would not be able to edit!
    let cfg_result = Config::load_config(api_key, profile_name, env_name, proj_name);
    if let Err(error) = cfg_result {
        let profile_info = if profile_name.is_some() {
            format!(" from profile '{}'", profile_name.unwrap())
        } else {
            "".to_string()
        };
        error_message(format!("Failed to load configuration{}.", profile_info,));
        help_message(format!(
            "The configuration ({}) can be edited with '{} config edit'.\nError details:\n{}",
            Config::filename(),
            cli::binary_name(),
            error.to_string()
        ));
        process::exit(26);
    }
    Config::init_global(cfg_result.unwrap());
    let rest_cfg = OpenApiConfig::from(Config::global());

    if let Some(matches) = matches.subcommand_matches("login") {
        process_login_command(matches, Config::global())?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("logout") {
        process_logout_command(matches, Config::global())?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("schema") {
        process_schema_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    // Check the basic config (api-key, server-url) -- don't worry about valid env/proj, yet
    check_config()?;

    if let Some(matches) = matches.subcommand_matches("users") {
        process_users_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("environments") {
        process_environment_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("projects") {
        process_project_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("integrations") {
        process_integrations_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("actions") {
        process_actions_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("audit-logs") {
        process_audit_log_command(matches, &rest_cfg)?;
        process::exit(0);
    }

    // Everything below here requires resolved environment/project values
    let resolved = resolve_ids(Config::global(), &rest_cfg)?;

    if let Some(matches) = matches.subcommand_matches("parameters") {
        process_parameters_command(matches, &rest_cfg, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("templates") {
        process_templates_command(matches, &rest_cfg, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        process_run_command(matches, &rest_cfg, &resolved)?;
    }

    Ok(())
}

#[cfg(test)]
mod main_test {
    use std::process::Command;

    use assert_cmd::prelude::*;
    use predicates::prelude::predicate::str::*;

    use crate::config::{CT_API_KEY, CT_PROFILE, CT_SERVER_URL};

    use super::*;

    fn cmd() -> Command {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();

        // Disable color output because it makes string matching hard in tests.
        cmd.env("NO_COLOR", "true");

        // Explicitly clear the API key so an individual dev's personal config isn't used for tests.
        cmd.env(CT_API_KEY, "");

        // Explicitly set the server to a bogus value that a server cannot to
        cmd.env(CT_SERVER_URL, "http://0.0.0.0:0");

        cmd
    }

    fn help_text() -> String {
        let mut help_cmd = cmd();
        help_cmd.arg("help").assert().success();

        let help_message = std::str::from_utf8(&help_cmd.output().unwrap().stdout)
            .unwrap()
            .to_string();

        help_message
    }

    #[test]
    fn requires_at_least_one_subcommand() {
        // Verify that invoking the CLI app without any arguments sets an error status code and
        // prints out the help message.
        let mut cmd = cmd();
        cmd.assert().failure().stderr(help_text());
    }

    #[test]
    fn completions_work_without_config() {
        let mut cmd = cmd();
        cmd.args(&["completions", "bash"]).assert().success();
    }

    #[test]
    fn completions_error_with_bad_shell_name() {
        let mut cmd = cmd();
        cmd.args(&["completions", "bad"])
            .assert()
            .failure()
            .stderr(contains("'bad' isn't a valid value"));
    }

    #[test]
    fn need_api_key() {
        let commands = &[
            vec!["parameters", "list"],
            vec!["environments", "list"],
            vec!["integrations", "list"],
            vec!["templates", "list"],
            vec!["--env", "non-default", "templates", "list"],
            vec!["run", "--command", "printenv"],
            vec!["run", "-c", "printenv"],
            vec!["run", "-s", "FOO=BAR", "--", "ls", "-lh", "/tmp"],
        ];
        for cmd_args in commands {
            println!("need_api_key test: {}", cmd_args.join(" "));
            let mut cmd = cmd();
            cmd.env(CT_API_KEY, "")
                .env(CT_PROFILE, "default")
                .args(cmd_args)
                .assert()
                .failure()
                .stderr(starts_with("The API key is missing."));
        }
    }

    #[test]
    fn missing_subcommands() {
        let commands = &[
            vec!["configuration"],
            vec!["projects"],
            vec!["environments"],
            vec!["integrations"],
            /*
            TODO: Rick Porter 3/2021: add more tests once we can get a valid environment, (e.g.
               environment, run)
            */
        ];
        for cmd_args in commands {
            println!("missing_subcommands test: {}", cmd_args.join(" "));
            let warn_msg = format!("WARN: No '{}' sub-command executed.", cmd_args[0]);
            let mut cmd = cmd();
            cmd.args(cmd_args)
                .env(CT_API_KEY, "dummy-key")
                .env(CT_PROFILE, "default")
                .assert()
                .stderr(starts_with(warn_msg));
        }
    }

    #[test]
    fn missing_profile() {
        let commands = &[vec!["projects"], vec!["environments"], vec!["integrations"]];
        for cmd_args in commands {
            let prof_name = "no-prof-with-this-name";
            println!("missing_profile test: {}", cmd_args.join(" "));
            let warn_msg = format!(
                "Profile '{}' does not exist in your configuration file",
                prof_name
            );
            let mut cmd = cmd();
            cmd.args(cmd_args)
                .env(CT_API_KEY, "dummy-key")
                .env(CT_PROFILE, prof_name)
                .assert()
                .stderr(contains(warn_msg));
        }
    }

    #[test]
    fn timedate_and_tag_parsing() {
        // full RFC2822
        let now = Utc::now();
        let input = now.to_rfc2822();
        let output = parse_datetime(Some(&input)).unwrap();
        assert_eq!(now.format("%FT%TZ").to_string(), output); // no fractional seconds
        assert_eq!(parse_tag(Some(&input)), None);

        // full RFC23339
        let now = Utc::now();
        let input = now.to_rfc3339();
        let output = parse_datetime(Some(&input)).unwrap();
        assert_eq!(now.format(ISO8601).to_string(), output);
        assert_eq!(parse_tag(Some(&input)), None);

        // ISO8601
        let input = Some("2021-07-27T18:34:23.270824Z");
        let expected = input.map(String::from);
        assert_eq!(parse_datetime(input), expected);
        assert_eq!(parse_tag(input), None);

        // ISO8601 - missing trailing Z
        let input = Some("2021-07-27T18:34:23.270824");
        let output = parse_datetime(input);
        assert_eq!(true, output.unwrap().contains(input.unwrap()));
        assert_eq!(parse_tag(input), None);

        // time only, without milliseconds
        let input = Some("02:04:08");
        let output = parse_datetime(input).unwrap();
        assert_eq!(true, output.contains("02:04:08"));
        assert_eq!(parse_tag(input), None);

        // time only, with milliseconds
        let input = Some("03:05:12.345");
        let output = parse_datetime(input).unwrap();
        assert_eq!(true, output.contains("T03:05:12.345Z"));
        assert_eq!(parse_tag(input), None);

        // full date (no time)
        let input = Some("2020-02-02");
        let output = parse_datetime(input).unwrap();
        assert_eq!(output, String::from("2020-02-02T00:00:00Z"));
        assert_eq!(parse_tag(input), None);

        // US date with slashes
        let input = Some("01/19/2021");
        let output = parse_datetime(input).unwrap();
        assert_eq!(output, String::from("2021-01-19T00:00:00Z"));
        assert_eq!(parse_tag(input), None);

        // US date with dashes
        let input = Some("01-19-2021");
        let output = parse_datetime(input).unwrap();
        assert_eq!(output, String::from("2021-01-19T00:00:00Z"));
        assert_eq!(parse_tag(input), None);

        // unfortunately, it lets this through too!
        let input = Some("this is bogus");
        let expected = input.map(String::from);
        assert_eq!(parse_datetime(input), None);
        assert_eq!(parse_tag(input), expected);

        // finally, no option given
        assert_eq!(parse_datetime(None), None);
        assert_eq!(parse_tag(None), None);
    }
}
