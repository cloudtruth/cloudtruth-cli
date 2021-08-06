extern crate rpassword;

mod cli;
mod config;
mod configuration;
mod crypto;
mod database;
mod environments;
mod integrations;
mod lib;
mod parameters;
mod projects;
mod run;
mod subprocess;
mod table;
mod templates;

use crate::config::env::ConfigEnv;
use crate::config::{Config, CT_PROFILE, DEFAULT_ENV_NAME};
use crate::configuration::process_config_command;
use crate::database::{Environments, Integrations, OpenApiConfig, Parameters, Projects, Templates};
use crate::environments::process_environment_command;
use crate::integrations::process_integrations_command;
use crate::parameters::process_parameters_command;
use crate::projects::process_project_command;
use crate::run::process_run_command;
use crate::subprocess::SubProcess;
use crate::templates::process_templates_command;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::io::{self, stdin, stdout, Write};
use std::process;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const REDACTED: &str = "*****";
const FILE_READ_ERR: &str = "Failed to read value from file.";

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
fn stderr_message(message: String, color: Color) -> Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(color));

    stderr.set_color(&color_spec)?;
    writeln!(&mut stderr, "{}", message)?;
    stderr.reset()?;
    Ok(())
}

/// Print the provided message to stderr in 'Yellow'.
fn warning_message(message: String) -> Result<()> {
    stderr_message(message, Color::Yellow)
}

/// Print the provided message to stderr in 'Red'.
fn error_message(message: String) -> Result<()> {
    stderr_message(message, Color::Red)
}

/// Print the provided message to stderr in 'Cyan'.
fn help_message(message: String) -> Result<()> {
    stderr_message(message, Color::Cyan)
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
                warning_message(message)?;
            }
        }

        let errors = issues.errors;
        if !errors.is_empty() {
            for err in errors {
                error_message(err.message)?;
                help_message(err.help_message)?;
            }
            process::exit(1)
        }
    }
    Ok(())
}

/// Add "WARN:" prefix to the message, and print it to stderr
fn warn_user(message: String) -> Result<()> {
    warning_message(format!("WARN: {}", message))
}

/// Simple method for standardizing the message when no sub-command is executed.
fn warn_missing_subcommand(command: &str) -> Result<()> {
    warn_user(format!("No '{}' sub-command executed.", command))
}

/// Method for standardizing message about list of warnings.
fn warn_unresolved_params(errors: &[String]) -> Result<()> {
    match errors.is_empty() {
        false => warning_message(format!(
            "Errors resolving parameters:\n{}\n",
            errors.join("\n")
        )),
        true => Ok(()),
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
fn user_confirm(message: String) -> bool {
    let max_tries = 3;
    let mut confirmed = false;

    for _ in 0..max_tries {
        let mut input = String::new();
        print!("{}? (y/n) ", message);
        stdout().flush().unwrap();
        let _ = stdin().read_line(&mut input);
        input = input.trim().to_string().to_lowercase();
        input.truncate(input.len());
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
        error_message(format!(
            "The '{}' environment could not be found in your account.",
            env,
        ))?;
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
            ))?;
            err = true;
        }
    } else {
        error_message("No project name was provided!".to_owned())?;
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

/// Process the 'completion' sub-command
fn process_completion_command(subcmd_args: &ArgMatches) {
    let shell = subcmd_args.value_of("SHELL").unwrap();

    cli::build_cli().gen_completions_to(
        cli::binary_name(),
        shell.parse().unwrap(),
        &mut io::stdout(),
    );
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
    Config::init_global(Config::load_config(
        api_key,
        profile_name,
        env_name,
        proj_name,
    )?);
    let rest_cfg = OpenApiConfig::from(Config::global());

    // Check the basic config (api-key, server-url) -- don't worry about valid env/proj, yet
    check_config()?;

    if let Some(matches) = matches.subcommand_matches("environments") {
        let environments = Environments::new();
        process_environment_command(matches, &rest_cfg, &environments)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("projects") {
        let projects = Projects::new();
        process_project_command(matches, &rest_cfg, &projects)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("integrations") {
        let integrations = Integrations::new();
        process_integrations_command(matches, &rest_cfg, &integrations)?;
        process::exit(0)
    }

    // Everything below here requires resolved environment/project values
    let resolved = resolve_ids(Config::global(), &rest_cfg)?;

    if let Some(matches) = matches.subcommand_matches("parameters") {
        let parameters = Parameters::new();
        process_parameters_command(matches, &rest_cfg, &parameters, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("templates") {
        let templates = Templates::new();
        process_templates_command(matches, &rest_cfg, &templates, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        let mut sub_proc = SubProcess::new();
        process_run_command(matches, &rest_cfg, &mut sub_proc, &resolved)?;
    }

    Ok(())
}

#[cfg(test)]
mod main_test {
    use crate::cli;
    use crate::config::{CT_API_KEY, CT_SERVER_URL};
    use assert_cmd::prelude::*;
    use predicates::prelude::predicate::str::*;
    use std::process::Command;

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
                .assert()
                .stderr(starts_with(warn_msg));
        }
    }
}
