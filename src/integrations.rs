use crate::cli::{FORMAT_OPT, LIST_SUBCMD, VALUES_FLAG};
use crate::database::{Integrations, OpenApiConfig};
use crate::table::Table;
use crate::{error_message, warn_missing_subcommand};
use clap::ArgMatches;
use color_eyre::eyre::Result;

fn proc_integ_explore(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
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
    Ok(())
}

fn proc_integ_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
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
    } else {
        warn_missing_subcommand("integrations")?;
    }
    Ok(())
}
