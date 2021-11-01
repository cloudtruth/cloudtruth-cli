use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, INTEGRATION_NAME_ARG,
    LIST_SUBCMD, PUSH_NAME_ARG, PUSH_SUBCMD, RENAME_OPT, SET_SUBCMD, SHOW_TIMES_FLAG, TASKS_SUBCMD,
};
use crate::database::{Integrations, OpenApiConfig};
use crate::table::Table;
use crate::{error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

fn integration_not_found_message(integ_name: &str) -> String {
    format!("No integration found for '{}'", integ_name)
}

fn integration_push_not_found_message(integ_name: &str, push_name: &str) -> String {
    format!(
        "No push integration found for '{}' in integration '{}'",
        push_name, integ_name
    )
}

fn proc_integ_explore(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let fqn = subcmd_args.value_of("FQN");
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let nodes = integrations.get_integration_nodes(rest_cfg, fqn)?;
    let indent = "  ";
    if nodes.is_empty() {
        if let Some(fqn) = fqn {
            error_message(format!("Nothing found for FQN '{}'!", fqn))?;
        } else {
            error_message("No integrations found.".to_string())?;
        }
    } else if !show_values {
        for node in nodes {
            println!("{}", node.name);
            for key in node.content_keys {
                println!("{}{{{{ {} }}}}", indent, key);
            }
        }
    } else {
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
    Ok(())
}

fn proc_integ_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let details = integrations.get_integration_details(rest_cfg)?;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    if details.is_empty() {
        println!("No integrations found");
    } else if !show_values {
        let list = details
            .iter()
            .map(|d| d.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut hdr = vec!["Name", "FQN", "Status", "Updated", "Description"];
        let mut properties = vec!["name", "fqn", "status", "status-time", "description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }
        let mut table = Table::new("integration");
        table.set_header(&hdr);
        for entry in details {
            table.add_row(entry.get_properties(&properties));
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_integ_refresh(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();

    let response_id = integrations.get_id(rest_cfg, integ_name)?;
    if let Some(integ_id) = response_id {
        integrations.refresh_connection(rest_cfg, &integ_id)?;
    } else {
        error_message(integration_not_found_message(integ_name))?;
        process::exit(32);
    }
    Ok(())
}

fn proc_integ_push_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();

    let response_id = integrations.get_id(rest_cfg, integ_name)?;
    if let Some(integ_id) = response_id {
        let response_id = integrations.get_push_id(rest_cfg, &integ_id, push_name)?;
        if let Some(push_id) = response_id {
            // NOTE: the server is responsible for checking if children exist
            let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
            if !confirmed {
                let msg = format!(
                    "Delete push '{}' from integetration '{}'",
                    push_name, integ_name
                );
                confirmed = user_confirm(msg, DEL_CONFIRM);
            }

            if !confirmed {
                warning_message(format!("Push '{}' not deleted from !", push_name))?;
            } else {
                integrations.delete_push(rest_cfg, &integ_id, &push_id)?;
                println!("Deleted push '{}' from '{}'", push_name, integ_name);
            }
        } else {
            warning_message(integration_push_not_found_message(integ_name, push_name))?;
        }
    } else {
        error_message(integration_not_found_message(integ_name))?;
        process::exit(30);
    }
    Ok(())
}

fn proc_integ_push_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    let response_id = integrations.get_id(rest_cfg, integ_name)?;
    if let Some(integ_id) = response_id {
        let pushes = integrations.get_push_list(rest_cfg, &integ_id)?;
        if pushes.is_empty() {
            println!("No pushes found for integration '{}'", integ_name);
        } else if !show_values {
            let list = pushes
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<String>>();
            println!("{}", list.join("\n"))
        } else {
            let mut hdr = vec!["Name", "Description", "Resource", "Last Task"];
            let mut properties = vec!["name", "description", "resource", "task-info"];
            if show_times {
                hdr.push("Created At");
                hdr.push("Modified At");
                properties.push("created-at");
                properties.push("modified-at");
            }

            let mut table = Table::new("integration-push");
            table.set_header(&hdr);
            for entry in pushes {
                table.add_row(entry.get_properties(&properties));
            }
            table.render(fmt)?;
        }
    } else {
        error_message(integration_not_found_message(integ_name))?;
        process::exit(30);
    }
    Ok(())
}

fn proc_integ_push_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let updated_name = subcmd_args.value_of(RENAME_OPT).unwrap_or(push_name);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let resource = subcmd_args.value_of("resource");
    let region = subcmd_args.value_of("region").unwrap();
    let service = subcmd_args.value_of("service").unwrap();

    let response_integ = integrations.get_id(rest_cfg, integ_name)?;
    if let Some(integ_id) = response_integ {
        let response_details = integrations.get_push_by_name(rest_cfg, &integ_id, push_name)?;
        if let Some(details) = response_details {
            // update code
            if subcmd_args.occurrences_of("region") > 0 {
                warning_message(format!(
                    "The --region is ignored for updates to '{}",
                    push_name
                ))?;
            }
            if subcmd_args.occurrences_of("service") > 0 {
                warning_message(format!(
                    "The --service is ignored for updates to '{}",
                    push_name
                ))?;
            }
            let updated_resource = resource.unwrap_or(&details.resource);
            integrations.update_push(
                rest_cfg,
                &integ_id,
                &details.id,
                updated_name,
                updated_resource,
                description,
            )?;
            println!(
                "Updated push '{}' in integration '{}'",
                updated_name, integ_name
            );
        } else {
            // create code
            if resource.is_none() {
                error_message("Must specify a resource value on create".to_string())?;
                process::exit(32);
            }
            integrations.create_push(
                rest_cfg,
                &integ_id,
                push_name,
                resource.unwrap(),
                region,
                service,
                description,
            )?;
            println!(
                "Created push '{}' in integration '{}'",
                push_name, integ_name
            );
        }
    } else {
        error_message(integration_not_found_message(integ_name))?;
        process::exit(30);
    }
    Ok(())
}

fn proc_integ_push_tasks(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    let response_id = integrations.get_id(rest_cfg, integ_name)?;
    if response_id.is_none() {
        error_message(integration_not_found_message(integ_name))?;
        process::exit(30);
    }

    let integ_id = response_id.unwrap();
    let response_id = integrations.get_push_id(rest_cfg, &integ_id, push_name)?;
    if response_id.is_none() {
        error_message(integration_push_not_found_message(integ_name, push_name))?;
        process::exit(31);
    }

    let push_id = response_id.unwrap();
    let tasks = integrations.get_push_tasks(rest_cfg, &integ_id, &push_id)?;
    if tasks.is_empty() {
        println!(
            "No push tasks found for push '{}' for integration '{}'",
            push_name, integ_name
        );
    } else if !show_values {
        let list = tasks
            .iter()
            .map(|d| d.reason.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut hdr = vec!["Reason", "State", "Status Info"];
        let mut properties = vec!["reason", "state", "errors"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }

        let mut table = Table::new("integration-push-task");
        table.set_header(&hdr);
        for entry in tasks {
            table.add_row(entry.get_properties(&properties));
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_integ_push_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_integ_push_delete(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_integ_push_list(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_integ_push_set(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TASKS_SUBCMD) {
        proc_integ_push_tasks(subcmd_args, rest_cfg, integrations)?;
    } else {
        warn_missing_subcommand("integrations pushes")?;
    }
    Ok(())
}

/// Process the 'integrations' sub-command
pub fn process_integrations_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("explore") {
        proc_integ_explore(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_integ_list(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(PUSH_SUBCMD) {
        proc_integ_push_command(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("refresh") {
        proc_integ_refresh(subcmd_args, rest_cfg, integrations)?;
    } else {
        warn_missing_subcommand("integrations")?;
    }
    Ok(())
}
