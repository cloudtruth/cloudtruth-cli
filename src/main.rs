mod cli;
mod config;
mod environments;
mod graphql;
mod parameters;
mod templates;

use crate::config::Config;
use crate::environments::Environments;
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

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = cli::build_cli().get_matches();

    let jwt = matches.value_of("api_key");
    Config::init_global(Config::load_config(jwt)?);

    let environments = Environments::new();

    let env = matches.value_of("env");
    if !environments.is_valid_environment_name(env)? {
        panic!(
            "The '{}' environment could not be found in your account.",
            env.unwrap_or("default")
        )
    }

    if let Some(matches) = matches.subcommand_matches("completions") {
        let shell = matches.value_of("SHELL").unwrap();

        cli::build_cli().gen_completions_to(
            cli::binary_name(),
            shell.parse().unwrap(),
            &mut io::stdout(),
        );
    }

    if let Some(matches) = matches.subcommand_matches("config") {
        if matches.subcommand_matches("edit").is_some() {
            Config::edit()?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("environments") {
        check_config()?;

        if matches.subcommand_matches("list").is_some() {
            let list = environments.get_environments()?;
            println!("{}", list.join("\n"))
        }
    }

    if let Some(matches) = matches.subcommand_matches("parameters") {
        check_config()?;

        let parameters = Parameters::new();

        if matches.subcommand_matches("list").is_some() {
            let list = parameters.get_parameters(environments.get_id(env)?)?;
            if list.is_empty() {
                println!("There are no parameters in your account.")
            } else {
                println!("{}", list.join("\n"))
            }
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let key = matches.value_of("KEY").unwrap();
            let parameter = parameters.get_body(env, key)?;

            if let Some(value) = parameter {
                println!("{}", value)
            } else {
                println!(
                    "Could not find a parameter with name '{}' in environment '{}'.",
                    key,
                    env.unwrap_or("default")
                );
            }
        } else if let Some(matches) = matches.subcommand_matches("set") {
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE");

            let updated_id = parameters.set_parameter(env, key, value)?;

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
        check_config()?;

        let templates = Templates::new();

        if matches.subcommand_matches("list").is_some() {
            let list = templates.get_templates()?;
            if list.is_empty() {
                println!("There are no templates in your account.")
            } else {
                println!("{}", list.join("\n"))
            }
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let template_name = matches.value_of("KEY").unwrap();
            let body = templates.get_body_by_name(env, template_name)?;

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

    #[test]
    fn completions_work_without_config() {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();
        cmd.args(&["completions", "bash"]).assert().success();
    }

    #[test]
    fn completions_error_with_bad_shell_name() {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();
        cmd.args(&["completions", "bad"])
            .assert()
            .failure()
            .stderr(contains("'bad' isn't a valid value"));
    }

    #[test]
    fn environments_commands_validate_config() {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();
        cmd.env("CT_API_KEY", "")
            .env("NO_COLOR", "true")
            .args(&["environments", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }

    #[test]
    fn parameters_commands_validate_config() {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();
        cmd.env("CT_API_KEY", "")
            .env("NO_COLOR", "true")
            .args(&["parameters", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }

    #[test]
    fn templates_commands_validate_config() {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();
        cmd.env("CT_API_KEY", "")
            .env("NO_COLOR", "true")
            .args(&["templates", "list"])
            .assert()
            .failure()
            .stderr(starts_with("The API key is missing."));
    }
}
