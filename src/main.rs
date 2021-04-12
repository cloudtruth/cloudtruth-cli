// Ignore the upper-case acronym warning from clippy for the whole module, since the GraphQLXxxx
// pattern tends to be liked by the team.
#![allow(clippy::upper_case_acronyms)]

mod graphql;

#[macro_use]
mod macros;

#[macro_use]
extern crate prettytable;

mod cli;
mod config;
mod environments;
mod parameters;
mod projects;
mod subprocess;
mod templates;

use crate::config::{Config, DEFAULT_ENV_NAME, DEFAULT_PROJ_NAME};
use crate::environments::Environments;
use crate::graphql::GraphQLError;
use crate::parameters::Parameters;
use crate::projects::{Projects, ProjectsIntf};
use crate::subprocess::{Inheritance, SubProcess, SubProcessIntf};
use crate::templates::Templates;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use prettytable::{format, Attr, Cell, Row, Table};
use std::io::{self, stdout, Write};
use std::process;
use std::str::FromStr;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const REDACTED: &str = "*****";

pub struct ResolvedIds {
    pub env_name: Option<String>,
    pub env_id: Option<String>,
    pub proj_name: Option<String>,
    pub proj_id: Option<String>,
}

fn stderr_message(message: String, color: Color) -> Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(color));

    stderr.set_color(&color_spec)?;
    writeln!(&mut stderr, "{}", message)?;
    stderr.reset()?;
    Ok(())
}

fn warning_message(message: String) -> Result<()> {
    stderr_message(message, Color::Yellow)
}

fn error_message(message: String) -> Result<()> {
    stderr_message(message, Color::Red)
}

fn help_message(message: String) -> Result<()> {
    stderr_message(message, Color::Cyan)
}

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

fn warn_user(message: String) -> Result<()> {
    warning_message(format!("WARN: {}", message))
}

fn warn_missing_subcommand(command: &str) -> Result<()> {
    warn_user(format!("No '{}' sub-command executed.", command))
}

/// Resolves the environment and project strings.
///
/// If either fails, it prints an error and exits.
/// On success, it returns a `ResolvedIds` structure that contains ids to avoid needing to resolve
/// the names again.
fn resolve_ids(org_id: Option<&str>, env: Option<&str>, proj: Option<&str>) -> Result<ResolvedIds> {
    // The `err` value is used to allow accumulation of multiple errors to the user.
    let mut err = false;
    let environments = Environments::new();
    let env_id = environments.get_id(org_id, env)?;
    if env_id.is_none() {
        error_message(format!(
            "The '{}' environment could not be found in your account.",
            env.unwrap_or(DEFAULT_ENV_NAME),
        ))?;
        err = true;
    }

    let projects = Projects::new();
    let proj_id = projects.get_id(org_id, proj)?;
    if proj_id.is_none() {
        error_message(format!(
            "The '{}' project could not be found in your account.",
            proj.unwrap_or(DEFAULT_PROJ_NAME)
        ))?;
        err = true;
    }

    // if any errors were encountered, exit with an error code
    if err {
        process::exit(2);
    }

    Ok(ResolvedIds {
        env_name: env.map(String::from),
        env_id,
        proj_name: proj.map(String::from),
        proj_id,
    })
}

fn process_run_command(
    org_id: Option<&str>,
    resolved: &ResolvedIds,
    subcmd_args: &ArgMatches,
) -> Result<()> {
    let mut sub_proc: SubProcess = SubProcess::new();
    let mut arguments: Vec<String>;
    let command: String;
    if subcmd_args.is_present("command") {
        command = subcmd_args.value_of("command").unwrap().to_string();
        arguments = vec![];
    } else if subcmd_args.is_present("arguments") {
        arguments = subcmd_args.values_of_lossy("arguments").unwrap();
        command = arguments.remove(0);
        if command.contains(' ') {
            warn_user("command contains spaces, and will likely fail.".to_string())?;
            let mut reformed = format!("{} {}", command, arguments.join(" "));
            reformed = reformed.replace("$", "\\$");
            println!("Try using 'cloudtruth run command \"{}\"'", reformed.trim());
        }
    } else {
        warn_missing_subcommand("run")?;
        process::exit(0);
    }

    // Setup the environment for the sub-process.
    let inherit = Inheritance::from_str(subcmd_args.value_of("inheritance").unwrap()).unwrap();
    let overrides = subcmd_args.values_of_lossy("set").unwrap_or_default();
    let removals = subcmd_args.values_of_lossy("remove").unwrap_or_default();
    let permissive = subcmd_args.is_present("permissive");
    sub_proc.set_environment(org_id, resolved, inherit, &overrides, &removals)?;
    if !permissive {
        sub_proc.remove_ct_app_vars();
    }
    sub_proc.run_command(command.as_str(), &arguments)?;

    Ok(())
}

fn process_project_command(
    org_id: Option<&str>,
    projects: &impl ProjectsIntf,
    subcmd_args: &ArgMatches,
) -> Result<()> {
    if subcmd_args.subcommand_matches("list").is_some() {
        let list = projects.get_project_names(org_id)?;
        println!("{}", list.join("\n"))
    } else {
        warn_missing_subcommand("projects")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let matches = cli::build_cli().get_matches();

    let api_key = matches.value_of("api_key");
    let profile_name = matches.value_of("profile");

    Config::init_global(Config::load_config(api_key, profile_name)?);

    if let Some(matches) = matches.subcommand_matches("completions") {
        let shell = matches.value_of("SHELL").unwrap();

        cli::build_cli().gen_completions_to(
            cli::binary_name(),
            shell.parse().unwrap(),
            &mut io::stdout(),
        );

        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("config") {
        if matches.subcommand_matches("edit").is_some() {
            Config::edit()?;
        } else if matches.subcommand_matches("list").is_some() {
            let profile_names = Config::get_profile_names()?;
            if profile_names.is_empty() {
                println!("No profiles exist in config.");
            } else {
                println!("{}", profile_names.join("\n"));
            }
        } else {
            warn_missing_subcommand("config")?;
        }

        process::exit(0)
    }

    let org_id: Option<&str> = None;

    // Check the basic config (api-key, server-url) -- don't worry about valid env/proj, yet
    check_config()?;

    if let Some(matches) = matches.subcommand_matches("environments") {
        let environments = Environments::new();
        if matches.subcommand_matches("list").is_some() {
            let list = environments.get_environment_names(org_id)?;
            println!("{}", list.join("\n"))
        } else {
            warn_missing_subcommand("environments")?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("projects") {
        let projects = Projects::new();
        process_project_command(org_id, &projects, matches)?;
    }

    // Everything below here requires resolved environment/project values
    let env = matches.value_of("env");
    let proj = matches.value_of("project");
    let resolved = resolve_ids(org_id, env, proj)?;

    if let Some(matches) = matches.subcommand_matches("parameters") {
        let parameters = Parameters::new();

        if let Some(matches) = matches.subcommand_matches("list") {
            let values = matches.is_present("values");
            if !values {
                let list = parameters.get_parameter_names(org_id, resolved.env_id.clone())?;
                if list.is_empty() {
                    println!("There are no parameters in your account.")
                } else {
                    println!("{}", list.join("\n"))
                }
            } else {
                let fmt = matches.value_of("format").unwrap();
                let ct_vars = parameters.get_parameter_details(org_id, resolved.env_id.clone())?;
                if ct_vars.is_empty() {
                    println!("No CloudTruth variables found!");
                } else {
                    let mut table = Table::new();
                    table.set_titles(Row::new(vec![
                        Cell::new("Name").with_style(Attr::Bold),
                        Cell::new("Value").with_style(Attr::Bold),
                        Cell::new("Source").with_style(Attr::Bold),
                        Cell::new("Description").with_style(Attr::Bold),
                    ]));
                    for entry in ct_vars {
                        let out_val = if entry.secret {
                            REDACTED.to_string()
                        } else {
                            entry.value
                        };
                        table.add_row(row![entry.key, out_val, entry.source, entry.description]);
                    }
                    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

                    if fmt == "csv" {
                        table.to_csv(stdout())?;
                    } else {
                        assert_eq!(fmt, "table");
                        table.printstd();
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let key = matches.value_of("KEY").unwrap();
            let parameter = parameters.get_body(org_id, env, key);

            if let Ok(parameter) = parameter {
                // Treat parameters without values set as if the value were simply empty, since
                // we need to display something sensible to the user.
                println!("{}", parameter.unwrap_or_else(|| "".to_string()));
            } else {
                match parameter.unwrap_err() {
                    GraphQLError::EnvironmentNotFoundError(name) => println!(
                        "The '{}' environment could not be found in your organization.",
                        name
                    ),
                    GraphQLError::ParameterNotFoundError(key) => println!(
                        "The parameter '{}' could not be found in your organization.",
                        key
                    ),
                    err => propagate_error!(err),
                };
            }
        } else if let Some(matches) = matches.subcommand_matches("set") {
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE");

            let updated_id = parameters.set_parameter(org_id, env, key, value)?;

            if updated_id.is_some() {
                println!(
                    "Successfully updated parameter '{}' in environment '{}'.",
                    key,
                    env.unwrap_or(DEFAULT_ENV_NAME)
                );
            } else {
                println!(
                    "Failed to update parameter '{}' in environment '{}'.",
                    key,
                    env.unwrap_or(DEFAULT_ENV_NAME)
                );
            }
        } else {
            warn_missing_subcommand("parameters")?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("templates") {
        let templates = Templates::new();

        if matches.subcommand_matches("list").is_some() {
            let list = templates.get_template_names(org_id)?;
            if list.is_empty() {
                println!("There are no templates in your account.")
            } else {
                println!("{}", list.join("\n"))
            }
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let template_name = matches.value_of("KEY").unwrap();
            let body = templates.get_body_by_name(org_id, env, template_name)?;

            if let Some(body) = body {
                println!("{}", body)
            } else {
                println!(
                    "Could not find a template with name '{}' in environment '{}'.",
                    template_name,
                    env.unwrap_or(DEFAULT_ENV_NAME)
                )
            }
        } else if let Some(matches) = matches.subcommand_matches("getit") {
            let starts_with = matches.value_of("starts_with");
            let ends_with = matches.value_of("ends_with");
            let contains = matches.value_of("contains");
            let template_format = matches.value_of("FORMAT").unwrap();
            let export = matches.is_present("export");
            let secrets = matches.is_present("secrets");
            let body = templates.get_body_by_implicit_name(
                org_id,
                env,
                starts_with,
                ends_with,
                contains,
                export,
                secrets,
                template_format,
            )?;

            if let Some(body) = body {
                println!("{}", body)
            } else {
                println!(
                    "Could not find a template with name '{}' in environment '{}'.",
                    template_format,
                    env.unwrap_or("default")
                )
            }
        } else {
            warn_missing_subcommand("templates")?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        process_run_command(org_id, &resolved, matches)?;
    }

    Ok(())
}

#[cfg(test)]
mod main_test {
    use crate::cli;
    use crate::config::{CT_API_KEY, CT_OLD_API_KEY, CT_SERVER_URL};
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
                .env(CT_OLD_API_KEY, "")
                .args(cmd_args)
                .assert()
                .failure()
                .stderr(starts_with("The API key is missing."));
        }
    }

    #[test]
    fn missing_subcommands() {
        let commands = &[
            vec!["config"],
            vec!["projects"],
            vec!["environments"],
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
