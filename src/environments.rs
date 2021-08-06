use crate::cli::{
    CONFIRM_FLAG, DELETE_SUBCMD, FORMAT_OPT, LIST_SUBCMD, NAME_ARG, RENAME_OPT, SET_SUBCMD,
    VALUES_FLAG,
};
use crate::config::DEFAULT_ENV_NAME;
use crate::database::{Environments, OpenApiConfig};
use crate::table::Table;
use crate::{error_message, user_confirm, warn_missing_subcommand, warning_message};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

/// Process the 'environment' sub-command
pub fn process_environment_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        let env_name = subcmd_args.value_of(NAME_ARG).unwrap();
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
        let env_name = subcmd_args.value_of(NAME_ARG).unwrap();
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
