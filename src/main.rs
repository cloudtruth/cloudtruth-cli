extern crate rpassword;

mod actions;
mod audit_logs;
mod backup;
mod cli;
mod config;
mod configuration;
mod database;
mod environments;
mod generate;
mod groups;
mod import;
mod installation;
mod integrations;
mod login;
mod logout;
mod parameters;
mod projects;
mod run;
mod schema;
mod subprocess;
mod table;
mod templates;
mod types;
mod users;
mod utils;
mod versions;

use crate::actions::process_actions_command;
use crate::audit_logs::process_audit_log_command;
use crate::backup::process_backup_command;
use crate::config::env::ConfigEnv;
use crate::config::{Action, Config, Updates, CT_PROFILE, DEFAULT_ENV_NAME};
use crate::configuration::process_config_command;
use crate::database::{OpenApiConfig, Resolver};
use crate::environments::process_environment_command;
use crate::generate::process_generate_command;
use crate::groups::process_groups_command;
use crate::import::process_import_command;
use crate::installation::{binary_version, get_latest_version, install_latest_version};
use crate::integrations::process_integrations_command;
use crate::login::process_login_command;
use crate::logout::process_logout_command;
use crate::parameters::process_parameters_command;
use crate::projects::process_project_command;
use crate::run::process_run_command;
use crate::schema::process_schema_command;
use crate::templates::process_templates_command;
use crate::types::process_parameter_type_command;
use crate::users::process_users_command;
use crate::utils::{error_message, help_message, warning_message};
use crate::versions::process_version_command;
use chrono::{Datelike, NaiveDate, Utc};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::io;
use std::process;
use utils::default;
use version_compare::Version;

/// Process the 'completion' sub-command
fn process_completion_command(subcmd_args: &ArgMatches) {
    let shell = subcmd_args.value_of("SHELL").unwrap();

    cli::build_cli().gen_completions_to(
        cli::binary_name(),
        shell.parse().unwrap(),
        &mut io::stdout(),
    );
}

/// Insures the basic configuration is valid (e.g. API key and server-url exist)
///
/// If there are errors, it will print the error/help and exit.
/// If only warnings happen, it will print the warning and keep going.
fn validate_config(config: &Config) -> Result<()> {
    if let Some(issues) = config.validate() {
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

/// This checks for newer CLI releases based on config and state
///
/// The Updates structure comes from the configuration file. If there is no configuration
/// file, there is no update checking. The update checking can be turned off in the config
/// file (if desired), but defaults to true.  The frequency of the checks and the action to
/// take on finding a newer version are also controlled in the config file.
fn check_updates(updates: &Updates) -> Result<()> {
    // NOTE: the next_update() returns None if the check is disabled...
    if let Some(next_update) = updates.next_update() {
        let now = Utc::today();
        let today = NaiveDate::from_ymd(now.year(), now.month(), now.day());
        if today >= next_update {
            let latest_str = get_latest_version();
            let latest_ver = Version::from(&latest_str).unwrap();
            let bin_str = binary_version();
            let bin_ver = Version::from(&bin_str).unwrap();

            if bin_ver < latest_ver {
                // NOTE: do not update last_checked date after we detect we're behind...
                match updates.action.unwrap_or_default() {
                    Action::Warn => {
                        warning_message(format!(
                            "Version {latest_ver} is available, running {bin_ver}"
                        ));
                    }
                    Action::Update => {
                        println!("Installing version {latest_ver}");
                        install_latest_version(false)?;
                    }
                    Action::Error => {
                        error_message(format!(
                            "Version {latest_ver} is available, running {bin_ver}"
                        ));
                        process::exit(50);
                    }
                }
            } else {
                let mut updated = *updates;
                updated.last_checked = Some(today);
                Config::set_updates(&updated)?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let profile_env = ConfigEnv::get_override(CT_PROFILE);
    let matches = cli::build_cli().get_matches();

    let api_key = matches.value_of("api_key");
    let profile_arg = matches.value_of("profile");
    let profile_name = matches.value_of("profile").or(profile_env.as_deref());
    let env_name = matches.value_of("env");
    let proj_name = matches.value_of("project");

    //====================================================
    // This section requires no configuration to be present
    if let Some(matches) = matches.subcommand_matches("completions") {
        process_completion_command(matches);
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("configuration") {
        process_config_command(matches, profile_arg, api_key, proj_name, env_name)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("versions") {
        process_version_command(matches)?;
        process::exit(0)
    }

    // check for updates based on the configuration (if any)
    if let Some(updates) = Config::load_updates()? {
        check_updates(&updates)?;
    }

    //====================================================
    // Check basic configuration stuff (e.g. server_url, and api_key), since these
    // commands will talk to the server
    let cfg_result = Config::load_config(api_key, profile_name, env_name, proj_name);
    if let Err(error) = cfg_result {
        let profile_info = profile_name.map_or(default(), |profile_name| {
            format!(" from profile '{profile_name}'")
        });
        error_message(format!("Failed to load configuration{profile_info}.",));
        help_message(format!(
            "The configuration ({}) can be edited with '{} config edit'.\nError details:\n{}",
            Config::filename(),
            cli::binary_name(),
            error
        ));
        process::exit(26);
    }
    let config = Config::init_global(cfg_result.unwrap());
    let rest_cfg = OpenApiConfig::from(config);

    if let Some(matches) = matches.subcommand_matches("login") {
        process_login_command(matches, config)?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("logout") {
        process_logout_command(matches, config)?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("schema") {
        process_schema_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    // Check the basic config (api-key, server-url) -- don't worry about valid env/proj, yet
    validate_config(config)?;

    if let Some(matches) = matches.subcommand_matches("users") {
        process_users_command(matches, &rest_cfg)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("groups") {
        process_groups_command(matches, &rest_cfg)?;
        process::exit(0);
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

    if let Some(matches) = matches.subcommand_matches("import") {
        process_import_command(matches, &rest_cfg)?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("parameter-types") {
        process_parameter_type_command(matches, &rest_cfg)?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("generate") {
        process_generate_command(matches, &rest_cfg)?;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("backup") {
        process_backup_command(matches, &rest_cfg)?;
        process::exit(0);
    }

    //====================================================
    // Everything below here requires resolved environment/project values
    let resolver = Resolver::new();
    let env_name = config.environment.as_deref().unwrap_or(DEFAULT_ENV_NAME);
    let proj_name = config.project.as_deref().unwrap_or_default();
    let resolved = resolver.resolve_ids(&rest_cfg, proj_name, env_name)?;

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
        cmd.args(["completions", "bash"]).assert().success();
    }

    #[test]
    fn completions_error_with_bad_shell_name() {
        let mut cmd = cmd();
        cmd.args(["completions", "bad"])
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
            let warn_msg =
                format!("Profile '{prof_name}' does not exist in your configuration file");
            let mut cmd = cmd();
            cmd.args(cmd_args)
                .env(CT_API_KEY, "dummy-key")
                .env(CT_PROFILE, prof_name)
                .assert()
                .stderr(contains(warn_msg));
        }
    }
}
