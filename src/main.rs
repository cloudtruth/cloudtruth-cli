extern crate rpassword;

mod cli;
mod config;
mod crypto;
mod environments;
mod integrations;
mod lib;
mod openapi;
mod parameters;
mod projects;
mod subprocess;
mod table;
mod templates;

use crate::cli::{
    CONFIRM_FLAG, DELETE_SUBCMD, FORMAT_OPT, GET_SUBCMD, LIST_SUBCMD, RENAME_OPT, SECRETS_FLAG,
    SET_SUBCMD, VALUES_FLAG,
};
use crate::config::env::ConfigEnv;
use crate::config::{Config, CT_PROFILE, DEFAULT_ENV_NAME};
use crate::environments::Environments;
use crate::integrations::Integrations;
use crate::openapi::OpenApiConfig;
use crate::parameters::{ParamExportFormat, ParamExportOptions, ParameterDetails, Parameters};
use crate::projects::Projects;
use crate::subprocess::{Inheritance, SubProcess};
use crate::table::Table;
use crate::templates::Templates;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use color_eyre::Report;
use rpassword::read_password;
use std::io::{self, stdin, stdout, Write};
use std::str::FromStr;
use std::{fs, process};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const REDACTED: &str = "********";

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
fn resolve_ids(config: &Config, rest_cfg: &mut OpenApiConfig) -> Result<ResolvedIds> {
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

/// Process the 'run' sub-command
fn process_run_command(
    subcmd_args: &ArgMatches,
    sub_proc: &mut SubProcess,
    resolved: &ResolvedIds,
) -> Result<()> {
    let mut arguments: Vec<String>;
    let command: String;
    if subcmd_args.is_present("command") {
        command = subcmd_args.value_of("command").unwrap().to_string();
        arguments = vec![];
    } else if subcmd_args.is_present("arguments") {
        arguments = subcmd_args.values_of_lossy("arguments").unwrap();
        command = arguments.remove(0);
        if command.contains(' ') {
            warn_user("command contains spaces, and may fail.".to_string())?;
            let mut reformed = format!("{} {}", command, arguments.join(" "));
            reformed = reformed.replace("$", "\\$");
            println!(
                "Try using 'cloudtruth run --command \"{}\"'",
                reformed.trim()
            );
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
    sub_proc.set_environment(resolved, inherit, &overrides, &removals)?;
    if !permissive {
        sub_proc.remove_ct_app_vars();
    }
    sub_proc.run_command(command.as_str(), &arguments)?;

    Ok(())
}

/// Process the 'project' sub-command
fn process_project_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &mut OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        let proj_name = subcmd_args.value_of("NAME").unwrap();
        let details = projects.get_details_by_name(rest_cfg, proj_name)?;

        if let Some(details) = details {
            // NOTE: the server is responsible for checking if children exist
            let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
            if !confirmed {
                confirmed = user_confirm(format!("Delete project '{}'", proj_name));
            }

            if !confirmed {
                warning_message(format!("Project '{}' not deleted!", proj_name))?;
            } else {
                projects.delete_project(rest_cfg, &details.id)?;
                println!("Deleted project '{}'", proj_name);
            }
        } else {
            warning_message(format!("Project '{}' does not exist!", proj_name))?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let details = projects.get_project_details(rest_cfg)?;
        if details.is_empty() {
            println!("No projects found.");
        } else if !subcmd_args.is_present(VALUES_FLAG) {
            let list = details
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"));
        } else {
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("project");
            table.set_header(&["Name", "Description"]);
            for entry in details {
                table.add_row(vec![entry.name, entry.description]);
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        let proj_name = subcmd_args.value_of("NAME").unwrap();
        let rename = subcmd_args.value_of(RENAME_OPT);
        let description = subcmd_args.value_of("description");
        let details = projects.get_details_by_name(rest_cfg, proj_name)?;

        if let Some(details) = details {
            if description.is_none() && rename.is_none() {
                warning_message(format!(
                    "Project '{}' not updated: no updated parameters provided",
                    proj_name
                ))?;
            } else {
                let name = rename.unwrap_or(&proj_name);
                projects.update_project(rest_cfg, name, &details.id, description)?;
                println!("Updated project '{}'", name);
            }
        } else {
            projects.create_project(rest_cfg, proj_name, description)?;
            println!("Created project '{}'", proj_name);
        }
    } else {
        warn_missing_subcommand("projects")?;
    }

    Ok(())
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

/// Process the 'config' sub-command
fn process_config_command(
    subcmd_args: &ArgMatches,
    profile_name: Option<&str>,
    api_key: Option<&str>,
    proj_name: Option<&str>,
    env_name: Option<&str>,
) -> Result<()> {
    if subcmd_args.subcommand_matches("edit").is_some() {
        Config::edit()?;
        let filepath = Config::config_file()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        println!("Edited {}", filepath);
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let details = Config::get_profile_details()?;
        if details.is_empty() {
            println!("No profiles exist in config.");
        } else if !subcmd_args.is_present(VALUES_FLAG) {
            let profile_names: Vec<String> = details.iter().map(|v| v.name.clone()).collect();
            println!("{}", profile_names.join("\n"));
        } else {
            let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("profile");
            table.set_header(&["Name", "API", "Environment", "Project", "Description"]);
            for entry in details {
                let mut api_value = "".to_string();
                if let Some(api_key) = entry.api_key {
                    if show_secrets {
                        api_value = api_key;
                    } else if !api_key.is_empty() {
                        api_value = REDACTED.to_string();
                    }
                }
                table.add_row(vec![
                    entry.name,
                    api_value,
                    entry.environment.unwrap_or_default(),
                    entry.project.unwrap_or_default(),
                    entry.description.unwrap_or_default(),
                ]);
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("current") {
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let show_extended = subcmd_args.is_present("extended");
        let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
        let values = Config::get_sources(profile_name, api_key, proj_name, env_name)?;

        let mut table = Table::new("profile");
        table.set_header(&["Parameter", "Value", "Source"]);
        for v in values {
            if show_extended || !v.extension {
                let val_str = if show_secrets || !v.secret || v.value.is_empty() {
                    v.value
                } else {
                    REDACTED.to_string()
                };
                table.add_row(vec![v.name, val_str, v.source]);
            }
        }

        table.render(fmt)?;
    } else {
        warn_missing_subcommand("configuration")?;
    }
    Ok(())
}

/// Process the 'environment' sub-command
fn process_environment_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &mut OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        let env_name = subcmd_args.value_of("NAME").unwrap();
        let details = environments.get_details_by_name(rest_cfg, env_name)?;

        if let Some(details) = details {
            // NOTE: the server is responsible for checking if children exist
            let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
            if !confirmed {
                confirmed = user_confirm(format!("Delete environment '{}'", env_name));
            }

            if !confirmed {
                warning_message(format!("Environment '{}' not deleted!", env_name))?;
            } else {
                environments.delete_environment(rest_cfg, details.id)?;
                println!("Deleted environment '{}'", env_name);
            }
        } else {
            warning_message(format!("Environment '{}' does not exist!", env_name))?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let details = environments.get_environment_details(rest_cfg)?;
        // NOTE: should always have at least the default environment
        if !subcmd_args.is_present(VALUES_FLAG) {
            let list = details
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"));
        } else {
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("environment");
            table.set_header(&["Name", "Parent", "Description"]);
            for entry in details {
                table.add_row(vec![entry.name, entry.parent_name, entry.description]);
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        let env_name = subcmd_args.value_of("NAME").unwrap();
        let parent_name = subcmd_args.value_of("parent");
        let description = subcmd_args.value_of("description");
        let rename = subcmd_args.value_of(RENAME_OPT);
        let details = environments.get_details_by_name(rest_cfg, env_name)?;

        if let Some(details) = details {
            if parent_name.is_some() && parent_name.unwrap() != details.parent_name.as_str() {
                error_message(format!(
                    "Environment '{}' parent cannot be updated.",
                    env_name
                ))?;
                process::exit(6);
            } else if description.is_none() && rename.is_none() {
                warning_message(format!(
                    "Environment '{}' not updated: no updated parameters provided",
                    env_name
                ))?;
            } else {
                let name = rename.unwrap_or(env_name);
                environments.update_environment(rest_cfg, &details.id, name, description)?;
                println!("Updated environment '{}'", name);
            }
        } else {
            let parent_name = parent_name.unwrap_or(DEFAULT_ENV_NAME);
            if let Some(parent_details) = environments.get_details_by_name(rest_cfg, parent_name)? {
                environments.create_environment(
                    rest_cfg,
                    env_name,
                    description,
                    parent_details.url.as_str(),
                )?;
                println!("Created environment '{}'", env_name);
            } else {
                error_message(format!("No parent environment '{}' found", parent_name))?;
                process::exit(5);
            }
        }
    } else {
        warn_missing_subcommand("environments")?;
    }
    Ok(())
}

/// Process the 'integrations' sub-command
fn process_integrations_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &mut OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("explore") {
        let fqn = subcmd_args.value_of("FQN");
        let nodes = integrations.get_integration_nodes(rest_cfg, fqn)?;
        let indent = "  ";
        if nodes.is_empty() {
            if let Some(fqn) = fqn {
                error_message(format!("Nothing found for FQN '{}'!", fqn))?;
            } else {
                error_message("No integrations found.".to_string())?;
            }
        } else if !subcmd_args.is_present("values") {
            for node in nodes {
                println!("{}", node.name);
                for key in node.content_keys {
                    println!("{}{{{{ {} }}}}", indent, key);
                }
            }
        } else {
            let fmt = subcmd_args.value_of("format").unwrap();
            let mut table = Table::new("integration");
            table.set_header(&["Name", "FQN"]);
            for node in nodes {
                // add the node itself
                table.add_row(vec![node.name, node.fqn.clone()]);
                for key in node.content_keys {
                    let entry_name = format!("{}{{{{ {} }}}}", indent, key);
                    table.add_row(vec![entry_name, node.fqn.clone()]);
                }
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let details = integrations.get_integration_details(rest_cfg)?;
        if details.is_empty() {
            println!("No integrations found");
        } else if !subcmd_args.is_present(VALUES_FLAG) {
            let list = details
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"))
        } else {
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("integration");
            table.set_header(&["Name", "FQN", "Status", "Updated", "Description"]);
            for entry in details {
                table.add_row(vec![
                    entry.name,
                    entry.fqn,
                    entry.status,
                    entry.status_time,
                    entry.description,
                ]);
            }
            table.render(fmt)?;
        }
    } else {
        warn_missing_subcommand("integrations")?;
    }
    Ok(())
}

/// Process the 'parameters' sub-command
fn process_parameters_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &mut OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let mut details =
            parameters.get_parameter_details(rest_cfg, proj_id, env_id, !show_secrets)?;
        let references = subcmd_args.is_present("dynamic");
        let qualifier = if references { "dynamic " } else { "" };
        if references {
            // when displaying dynamic parameters, only show the dynamic ones
            details.retain(|x| x.dynamic)
        }

        if details.is_empty() {
            println!(
                "No {}parameters found in project {}",
                qualifier,
                resolved.project_display_name()
            );
        } else if !subcmd_args.is_present(VALUES_FLAG) {
            let list = details
                .iter()
                .map(|d| d.key.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"))
        } else {
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("parameter");

            if !references {
                table.set_header(&["Name", "Value", "Source", "Type", "Secret", "Description"]);
            } else {
                table.set_header(&["Name", "FQN", "JMES"]);
            }

            for entry in details {
                if !references {
                    let type_str = if entry.dynamic { "dynamic" } else { "static" };
                    let secret_str = if entry.secret { "true" } else { "false" };
                    table.add_row(vec![
                        entry.key,
                        entry.value,
                        entry.env_name,
                        type_str.to_string(),
                        secret_str.to_string(),
                        entry.description,
                    ]);
                } else {
                    table.add_row(vec![entry.key, entry.fqn, entry.jmes_path]);
                }
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        let key = subcmd_args.value_of("KEY").unwrap();
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let parameter = parameters.get_details_by_name(rest_cfg, proj_id, env_id, key);

        if let Ok(details) = parameter {
            // Treat parameters without values set as if the value were simply empty, since
            // we need to display something sensible to the user.
            let mut param_value = "".to_string();
            if let Some(param) = details {
                param_value = param.value;
            }
            println!("{}", param_value);
        } else {
            println!(
                "The parameter '{}' could not be found in your organization.",
                key
            );
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        let key_name = subcmd_args.value_of("KEY").unwrap();
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let prompt_user = subcmd_args.is_present("prompt");
        let filename = subcmd_args.value_of("input-file");
        let fqn = subcmd_args.value_of("FQN");
        let jmes_path = subcmd_args.value_of("JMES");
        let mut value = subcmd_args.value_of("value");
        let val_str: String;
        let description = subcmd_args.value_of("description");
        let rename = subcmd_args.value_of(RENAME_OPT);
        let final_name = rename.unwrap_or(key_name);
        let mut param_added = false;
        let secret: Option<bool> = match subcmd_args.value_of("secret") {
            Some("false") => Some(false),
            Some("true") => Some(true),
            _ => None,
        };

        // make sure the user did not over-specify
        if (jmes_path.is_some() || fqn.is_some())
            && (value.is_some() || prompt_user || filename.is_some())
        {
            error_message(
                concat!(
                    "Conflicting arguments: cannot specify prompt/input-file/value, ",
                    "and fqn/jmes-path"
                )
                .to_string(),
            )?;
            process::exit(7);
        }

        // if user asked to be prompted
        if prompt_user {
            println!("Please enter the '{}' value: ", key_name);
            val_str = read_password()?;
            value = Some(val_str.as_str());
        } else if let Some(filename) = filename {
            val_str = fs::read_to_string(filename).expect("Failed to read value from file.");
            value = Some(val_str.as_str());
        }

        // make sure there is at least one parameter to updated
        if description.is_none()
            && secret.is_none()
            && value.is_none()
            && jmes_path.is_none()
            && fqn.is_none()
            && rename.is_none()
        {
            warn_user(
                concat!(
                    "Nothing changed. Please provide at least one of: ",
                    "description, rename, secret, or value/fqn/jmes-path."
                )
                .to_string(),
            )?;
        } else {
            // get the original values, so that is not lost
            let mut updated: ParameterDetails;
            if let Some(original) =
                parameters.get_details_by_name(rest_cfg, proj_id, env_id, key_name)?
            {
                // only update if there is something to update
                if description.is_some() || secret.is_some() || rename.is_some() {
                    updated = parameters.update_parameter(
                        rest_cfg,
                        proj_id,
                        &original.id,
                        &final_name,
                        description,
                        secret,
                    )?;
                    // copy a few fields to insure we detect the correct environment
                    updated.val_id = original.val_id;
                    updated.env_url = original.env_url;
                    updated.env_name = original.env_name;
                } else {
                    // nothing to update here, but need to copy details
                    updated = original;
                }
            } else {
                param_added = true;
                updated = parameters.create_parameter(
                    rest_cfg,
                    proj_id,
                    key_name,
                    description,
                    secret,
                )?;
            }

            // don't do anything if there's nothing to do
            if value.is_some() || fqn.is_some() || jmes_path.is_some() {
                let param_id = updated.id.as_str();
                // if any existing environment does not match the desired environment
                if !updated.env_url.contains(env_id) {
                    let value_add_result = parameters.create_parameter_value(
                        rest_cfg, proj_id, env_id, param_id, value, fqn, jmes_path,
                    );
                    if let Err(err) = value_add_result {
                        if param_added {
                            let _ = parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id);
                        }
                        return Err(Report::new(err));
                    }
                } else {
                    parameters.update_parameter_value(
                        rest_cfg,
                        proj_id,
                        param_id,
                        &updated.val_id,
                        value,
                        fqn,
                        jmes_path,
                    )?;
                }
            }
            println!(
                "Successfully updated parameter '{}' in project '{}' for environment '{}'.",
                final_name,
                resolved.project_display_name(),
                resolved.environment_display_name(),
            );
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        let key_name = subcmd_args.value_of("KEY").unwrap();
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let result = parameters.delete_parameter(rest_cfg, proj_id, env_id, key_name);
        match result {
            Ok(Some(_)) => {
                println!(
                    "Successfully removed parameter '{}' from project '{}'.",
                    key_name,
                    resolved.project_display_name(),
                );
            }
            Ok(None) => {
                println!(
                    "Did not find parameter '{}' to delete from project '{}'.",
                    key_name,
                    resolved.project_display_name(),
                )
            }
            _ => {
                println!(
                    "Failed to remove parameter '{}' from project '{}'.",
                    key_name,
                    resolved.project_display_name(),
                );
            }
        };
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("export") {
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let starts_with = subcmd_args.value_of("starts_with");
        let ends_with = subcmd_args.value_of("ends_with");
        let contains = subcmd_args.value_of("contains");
        let template_format = subcmd_args.value_of("FORMAT").unwrap();
        let export = subcmd_args.is_present("export");
        let secrets = subcmd_args.is_present(SECRETS_FLAG);
        let options = ParamExportOptions {
            format: ParamExportFormat::from_str(template_format).unwrap(),
            starts_with: starts_with.map(|s| s.to_string()),
            ends_with: ends_with.map(|s| s.to_string()),
            contains: contains.map(|s| s.to_string()),
            export: Some(export),
            secrets: Some(secrets),
        };
        let body = parameters.export_parameters(rest_cfg, proj_id, env_id, options)?;

        if let Some(body) = body {
            println!("{}", body)
        } else {
            println!(
                "Could not export parameters format '{}' from project '{}' in environment '{}'.",
                template_format,
                resolved.project_display_name(),
                resolved.environment_display_name()
            )
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("unset") {
        let key_name = subcmd_args.value_of("KEY").unwrap();
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let result = parameters.delete_parameter_value(rest_cfg, proj_id, env_id, key_name);
        match result {
            Ok(Some(_)) => {
                println!(
                    "Successfully removed parameter value '{}' from project '{}' for environment '{}'.",
                    key_name,
                    resolved.project_display_name(),
                    resolved.environment_display_name()
                );
            }
            Ok(None) => {
                println!(
                    "Did not find parameter value '{}' to delete from project '{}' for environment '{}'.",
                    key_name,
                    resolved.project_display_name(),
                    resolved.environment_display_name()
                )
            }
            _ => {
                println!(
                    "Failed to remove parameter value '{}' from project '{}' for environment '{}'.",
                    key_name,
                    resolved.project_display_name(),
                    resolved.environment_display_name()
                );
            }
        };
    } else {
        warn_missing_subcommand("parameters")?;
    }
    Ok(())
}

/// Process the 'templates' sub-command
fn process_templates_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &mut OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let proj_name = resolved.project_display_name();
        let proj_id = resolved.project_id();
        let details = templates.get_template_details(rest_cfg, proj_id)?;
        if details.is_empty() {
            println!("There are no templates in project `{}`.", proj_name);
        } else if !subcmd_args.is_present(VALUES_FLAG) {
            let list = details
                .iter()
                .map(|n| n.name.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"))
        } else {
            let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
            let mut table = Table::new("template");
            table.set_header(&["Name", "Description"]);
            for entry in details {
                table.add_row(vec![entry.name, entry.description]);
            }
            table.render(fmt)?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        let proj_name = resolved.project_display_name();
        let env_name = resolved.environment_display_name();
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let template_name = subcmd_args.value_of("KEY").unwrap();
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let body =
            templates.get_body_by_name(rest_cfg, proj_id, env_id, template_name, show_secrets)?;

        if let Some(body) = body {
            println!("{}", body)
        } else {
            println!(
                "Could not find a template with name '{}' in project '{}' environment '{}'.",
                template_name, proj_name, env_name
            )
        }
    } else {
        warn_missing_subcommand("templates")?;
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
    let mut rest_cfg = OpenApiConfig::from(Config::global());

    // Check the basic config (api-key, server-url) -- don't worry about valid env/proj, yet
    check_config()?;

    if let Some(matches) = matches.subcommand_matches("environments") {
        let environments = Environments::new();
        process_environment_command(matches, &mut rest_cfg, &environments)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("projects") {
        let projects = Projects::new();
        process_project_command(matches, &mut rest_cfg, &projects)?;
        process::exit(0)
    }

    if let Some(matches) = matches.subcommand_matches("integrations") {
        let integrations = Integrations::new();
        process_integrations_command(matches, &mut rest_cfg, &integrations)?;
        process::exit(0)
    }

    // Everything below here requires resolved environment/project values
    let resolved = resolve_ids(Config::global(), &mut rest_cfg)?;

    if let Some(matches) = matches.subcommand_matches("parameters") {
        let parameters = Parameters::new();
        process_parameters_command(matches, &mut rest_cfg, &parameters, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("templates") {
        let templates = Templates::new();
        process_templates_command(matches, &mut rest_cfg, &templates, &resolved)?;
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        let parameters = Parameters::new();
        let ct_vars = parameters.get_parameter_values(
            &mut rest_cfg,
            resolved.project_id(),
            resolved.environment_id(),
        )?;
        let mut sub_proc = SubProcess::new(ct_vars);
        process_run_command(matches, &mut sub_proc, &resolved)?;
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
