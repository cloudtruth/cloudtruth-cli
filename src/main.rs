mod cli;
mod config;
mod environments;
mod graphql;
mod parameters;
mod templates;

#[macro_use]
mod macros;

use crate::config::Config;
use crate::environments::Environments;
use crate::graphql::GraphQLError;
use crate::parameters::Parameters;
use crate::templates::Templates;
use color_eyre::eyre::Result;
use std::io::Write;
use std::{io, process};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn check_config() -> Result<()> {
    if let Some(errors) = Config::global().validate() {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
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

        process::exit(1)
    }

    Ok(())
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
            env.unwrap_or("default")
        )
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = cli::build_cli().get_matches();

    let api_key = matches.value_of("api_key");
    Config::init_global(Config::load_config(api_key)?);

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
                    env.unwrap_or("default")
                );
            } else {
                println!(
                    "Failed to update parameter '{}' in environment '{}'.",
                    key,
                    env.unwrap_or("default")
                );
            }
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
                    env.unwrap_or("default")
                )
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cli;
    use assert_cmd::prelude::*;
    use predicates::prelude::predicate::str::*;
    use std::process::Command;

    fn cmd() -> Command {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();

        // Disable color output because it makes string matching hard in tests.
        cmd.env("NO_COLOR", "true");

        // Explicitly clear the API key so an individual dev's personal config isn't used for tests.
        cmd.env("CT_API_KEY", "");

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
    fn environments_commands_validate_config() {
        let mut cmd = cmd();
        cmd.env("CT_API_KEY", "")
            .args(&["environments", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }

    #[test]
    fn parameters_commands_validate_config() {
        let mut cmd = cmd();
        cmd.env("CT_API_KEY", "")
            .args(&["parameters", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }

    #[test]
    fn templates_commands_validate_config() {
        let mut cmd = cmd();
        cmd.env("CT_API_KEY", "")
            .args(&["templates", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }

    #[test]
    fn environment_validation_also_validates_config() {
        let mut cmd = cmd();
        cmd.env("CT_API_KEY", "")
            .args(&["--env", "non-default", "templates", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }
}
