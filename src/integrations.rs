use crate::cli::{
    show_values, FORMAT_OPT, GET_SUBCMD, INTEGRATION_NAME_ARG, LIST_SUBCMD, RAW_FLAG,
    SHOW_TIMES_FLAG,
};
use crate::database::{Integrations, OpenApiConfig};
use crate::table::Table;
use crate::{error_message, warn_missing_subcommand, warning_message};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::process;

pub fn integration_not_found_message(integ_name: &str) -> String {
    format!("Integration '{}' not found", integ_name)
}

fn proc_integ_explore(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let fqn = subcmd_args.value_of("FQN");
    let show_raw = subcmd_args.is_present(RAW_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let nodes = integrations.get_integration_nodes(rest_cfg, fqn)?;
    let indent = "  ";
    if nodes.is_empty() {
        if let Some(fqn) = fqn {
            error_message(format!("Nothing found for FQN '{}'!", fqn));
        } else {
            error_message("No integrations found.".to_string());
        }
    } else if show_raw {
        if nodes.len() > 1 {
            warning_message(format!(
                "Raw content only works for a single file -- specified FQN has {} nodes.",
                nodes.len()
            ));
        } else if nodes[0].node_type != "File" {
            warning_message(format!(
                "Raw content only works for a single file -- specified FQN is {} type.",
                nodes[0].node_type.to_lowercase()
            ));
        } else {
            println!("{}", &nodes[0].content_data);
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

fn proc_integ_get(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG).unwrap();

    let integ_resp = integrations.get_details_by_name(rest_cfg, integ_name)?;
    if let Some(details) = integ_resp {
        printdoc!(
            r#"
            Name: {}
            Provider: {}
            FQN: {}
            Description: {}
            ID: {}
            Created At: {}
            Modified At: {}
            Status:
              Value: {}
              Details: {}
              Updated At: {}
            "#,
            details.name,
            details.provider,
            details.fqn,
            details.description,
            details.id,
            details.created_at,
            details.modified_at,
            details.status,
            details.status_detail,
            details.status_time,
        );
    } else {
        error_message(integration_not_found_message(integ_name));
        process::exit(32);
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

    let integ_resp = integrations.get_id(rest_cfg, integ_name)?;
    if let Some(integ_id) = integ_resp {
        integrations.refresh_connection(rest_cfg, &integ_id)?;
        println!("Refreshed integration '{}'", integ_name);
    } else {
        error_message(integration_not_found_message(integ_name));
        process::exit(32);
    }
    Ok(())
}

/// Process the 'integrations' sub-command
pub fn process_integrations_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
) -> Result<()> {
    let integrations = Integrations::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("explore") {
        proc_integ_explore(subcmd_args, rest_cfg, &integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_integ_get(subcmd_args, rest_cfg, &integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_integ_list(subcmd_args, rest_cfg, &integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("refresh") {
        proc_integ_refresh(subcmd_args, rest_cfg, &integrations)?;
    } else {
        warn_missing_subcommand("integrations");
    }
    Ok(())
}
