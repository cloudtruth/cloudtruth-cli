mod graphql;

#[macro_use]
mod macros;

#[macro_use]
extern crate prettytable;

mod cli;
mod config;
mod environments;
mod parameters;
mod subprocess;
mod templates;

use crate::config::Config;
use crate::config::DEFAULT_ENV_NAME;
use crate::environments::Environments;
use crate::graphql::GraphQLError;
use crate::parameters::Parameters;
use crate::subprocess::{Inheritance, SubProcess, SubProcessIntf};
use crate::templates::Templates;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use prettytable::{format, Attr, Cell, Row, Table};
use std::io::{self, stdout, Write};
use std::process;
use std::str::FromStr;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const REDACTED: &str = "REDACTED";

fn check_config() -> Result<()> {
    if let Some(issues) = Config::global().validate() {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);

        // print the warnings first, so the user sees them (even when errors are present)
        let warnings = issues.warnings;
        if !warnings.is_empty() {
            let mut warning_color_spec = ColorSpec::new();
            warning_color_spec.set_fg(Some(Color::Yellow));
            stderr.set_color(&warning_color_spec)?;

            for message in warnings {
                writeln!(&mut stderr, "{}", message)?;
            }
            stderr.reset()?;
        }

        let errors = issues.errors;
        if !errors.is_empty() {
            let mut error_color_spec = ColorSpec::new();
            let mut help_color_spec = ColorSpec::new();

            error_color_spec.set_fg(Some(Color::Red));
            help_color_spec.set_fg(Some(Color::Cyan));

            for message in errors {
                stderr.set_color(&error_color_spec)?;
                writeln!(&mut stderr, "{}", message.message)?;

                stderr.set_color(&help_color_spec)?;
                writeln!(&mut stderr, "{}", message.help_message)?;
            }
            stderr.reset()?;

            process::exit(1)
        }
    }

    Ok(())
}

fn warn_user(message: String) {
    println!("WARN: {}", message);
}

fn warn_missing_subcommand(command: &str) {
    warn_user(format!("No '{}' sub-command executed.", command));
}

fn check_valid_env(
    org_id: Option<&str>,
    env: Option<&str>,
    environments: &Environments,
) -> Result<()> {
    check_config()?;

    if !environments.is_valid_environment_name(org_id, env)? {
        panic!(
            "The '{}' environment could not be found in your account.",
            env.unwrap_or(DEFAULT_ENV_NAME)
        )
    }

    Ok(())
}

fn process_run_command(
    org_id: Option<&str>,
    env: Option<&str>,
    environments: &Environments,
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
            warn_user("command contains spaces, and will likely fail.".to_string());
            let mut reformed = format!("{} {}", command, arguments.join(" "));
            reformed = reformed.replace("$", "\\$");
            println!("Try using 'cloudtruth run command \"{}\"'", reformed.trim());
        }
    } else {
        warn_missing_subcommand("run");
        process::exit(0);
    }

    // Setup the environment for the sub-process.
    let inherit = Inheritance::from_str(subcmd_args.value_of("inheritance").unwrap()).unwrap();
    let overrides = subcmd_args.values_of_lossy("set").unwrap_or_default();
    let removals = subcmd_args.values_of_lossy("remove").unwrap_or_default();
    sub_proc.set_environment(org_id, env, environments, inherit, &overrides, &removals)?;
    sub_proc.run_command(command.as_str(), &arguments)?;

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
        } else {
            warn_missing_subcommand("config");
        }

        process::exit(0)
    }

    let environments = Environments::new();
    let env = matches.value_of("env");
    let org_id: Option<&str> = None;

    if let Some(matches) = matches.subcommand_matches("environments") {
        check_valid_env(org_id, env, &environments)?;

        if matches.subcommand_matches("list").is_some() {
            let list = environments.get_environment_names(org_id)?;
            println!("{}", list.join("\n"))
        } else {
            warn_missing_subcommand("environments");
        }
    }

    if let Some(matches) = matches.subcommand_matches("parameters") {
        check_valid_env(org_id, env, &environments)?;

        let parameters = Parameters::new();

        if matches.subcommand_matches("list").is_some() {
            let list = parameters.get_parameter_names(org_id, environments.get_id(org_id, env)?)?;
            if list.is_empty() {
                println!("There are no parameters in your account.")
            } else {
                println!("{}", list.join("\n"))
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
        } else if let Some(matches) = matches.subcommand_matches("show") {
            let fmt = matches.value_of("format").unwrap();
            let env_id = environments.get_id(org_id, env)?;
            let ct_vars = parameters.get_parameter_details(org_id, env_id)?;
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
        } else {
            warn_missing_subcommand("parameters");
        }
    }

    if let Some(matches) = matches.subcommand_matches("templates") {
        check_valid_env(org_id, env, &environments)?;

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
            let template_name = matches.value_of("NAME").unwrap();
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
                template_name,
            )?;

            if let Some(body) = body {
                println!("{}", body)
            } else {
                println!(
                    "Could not find a template with name '{}' in environment '{}'.",
                    template_name,
                    env.unwrap_or("default")
                )
            }
        } else {
            warn_missing_subcommand("templates");
        }
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        check_valid_env(org_id, env, &environments)?;
        process_run_command(org_id, env, &environments, matches)?;
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
            /* TODO: Rick Porter 3/2021: add more tests once we can get a valid environment, (e.g.
               environment, run)
            */
        ];
        for cmd_args in commands {
            println!("missing_subcommands test: {}", cmd_args.join(" "));
            let warn_msg = format!("WARN: No '{}' sub-command executed.", cmd_args[0]);
            let mut cmd = cmd();
            cmd.args(cmd_args)
                .assert()
                .success()
                .stdout(starts_with(warn_msg));
        }
    }
}
