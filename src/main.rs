mod graphql;

#[macro_use]
mod macros;

mod cli;
mod config;
mod environments;
mod parameters;
mod templates;

use crate::config::Config;
use crate::config::{DEFAULT_ENV_NAME, ENV_VAR_PREFIX};
use crate::environments::Environments;
use crate::graphql::GraphQLError;
use crate::parameters::Parameters;
use crate::templates::Templates;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use std::{env, process};
use subprocess::Exec;
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

fn current_env() -> HashMap<String, String> {
    // Create a HashMap from the current set of environment variables (excluding a few).
    let exclude = ["PS1", "TERM", "HOME"];

    env::vars()
        .filter(|(ref k, _)| !exclude.contains(&k.as_str()))
        .collect()
}

fn get_ct_vars(
    org_id: Option<&str>,
    env: Option<&str>,
    environments: &Environments,
) -> Result<HashMap<String, String>> {
    // Create a HashMap with all the CloudTruth environment values for this environment.
    let mut ct_vars = HashMap::new();
    let parameters = Parameters::new();
    let env_id = environments.get_id(org_id, env)?;
    let list = parameters.get_parameter_names(org_id, env_id)?;
    for key in list.iter() {
        let parameter = parameters.get_body(org_id, env, key)?;
        // Put the key/value pair into the environment
        let value = parameter.unwrap_or_else(|| "".to_string());
        ct_vars.insert(key.to_string(), value);
    }

    Ok(ct_vars)
}

fn process_overrides(overrides: Vec<String>) -> HashMap<String, String> {
    // Create HashMap with all the user-provided overrides.
    let mut over_vars = HashMap::new();
    for arg_val in overrides {
        let temp: Vec<&str> = arg_val.splitn(2, '=').collect();
        if temp.len() != 2 {
            warn_user(format!("Ignoring {} due to  no '='", arg_val));
            continue;
        }
        over_vars.insert(temp[0].to_string(), temp[1].to_string());
    }

    over_vars
}

fn process_run_command(
    org_id: Option<&str>,
    env: Option<&str>,
    environments: &Environments,
    subcmd_args: &ArgMatches,
) -> Result<()> {
    let mut sub_proc: Exec;
    if let Some(command) = subcmd_args.value_of("command") {
        sub_proc = Exec::shell(command);
    } else if let Some(mut arguments) = subcmd_args.values_of_lossy("arguments") {
        let command = arguments.remove(0);
        if command.contains(' ') {
            warn_user("command contains spaces, and will likely fail.".to_string());
            let mut reformed = format!("{} {}", command, arguments.join(" "));
            reformed = reformed.replace("$", "\\$");
            println!("Try using 'cloudtruth run command \"{}\"'", reformed.trim());
        }
        sub_proc = Exec::cmd(command).args(&arguments);
    } else {
        warn_missing_subcommand("run");
        process::exit(0);
    }

    // Setup the environment for the sub-process.
    let preserve = subcmd_args.is_present("preserve");
    let overrides = subcmd_args.values_of_lossy("set").unwrap_or_default();
    let removals = subcmd_args.values_of_lossy("remove").unwrap_or_default();
    let mut env_vars = if !preserve {
        HashMap::new()
    } else {
        current_env()
    };

    // Add breadcrumbs about which environment.
    env_vars.insert(
        format!("{}ENV", ENV_VAR_PREFIX),
        env.unwrap_or(DEFAULT_ENV_NAME).to_string(),
    );

    // Add in the items from the CloudTruth environment, and overrides.
    env_vars.extend(get_ct_vars(org_id, env, environments)?);
    env_vars.extend(process_overrides(overrides));

    // Remove the specified values.
    for r in removals {
        env_vars.remove(r.as_str());
    }

    // Common setup for the subprocess. By default, it streams stdin/stdout/stderr to parent.
    sub_proc = sub_proc.env_clear();
    for (key, value) in env_vars {
        sub_proc = sub_proc.env(key, value);
    }

    // Run the process and wait for the result.
    sub_proc.join()?;

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
    use crate::config::CT_API_KEY;
    use assert_cmd::prelude::*;
    use predicates::prelude::predicate::str::*;
    use std::process::Command;

    fn cmd() -> Command {
        let mut cmd = Command::cargo_bin(cli::binary_name()).unwrap();

        // Disable color output because it makes string matching hard in tests.
        cmd.env("NO_COLOR", "true");

        // Explicitly clear the API key so an individual dev's personal config isn't used for tests.
        cmd.env(CT_API_KEY, "");

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
            vec!["run", "-s", "", "--", "ls", "-lh", "/tmp"],
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
