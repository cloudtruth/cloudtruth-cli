use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, LIST_SUBCMD, NAME_ARG,
    PARENT_ARG, RENAME_OPT, SET_SUBCMD, SHOW_TIMES_FLAG, TREE_SUBCMD,
};
use crate::database::{OpenApiConfig, TypeDetails, Types};
use crate::table::Table;
use crate::{error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

fn proc_param_type_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let type_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let type_id = types.get_id(rest_cfg, type_name)?;

    if let Some(type_id) = type_id {
        // NOTE: the server is responsible for checking if children exist
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(
                format!("Delete parameter type '{}'", type_name),
                DEL_CONFIRM,
            );
        }

        if !confirmed {
            warning_message(format!("Parameter type '{}' not deleted!", type_name));
        } else {
            types.delete_type(rest_cfg, &type_id)?;
            println!("Deleted parameter type '{}'", type_name);
        }
    } else {
        warning_message(format!("Parameter type '{}' does not exist!", type_name));
    }
    Ok(())
}

fn proc_param_type_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let details = types.get_type_details(rest_cfg)?;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    if details.is_empty() {
        println!("No parameter types found.");
    } else if !show_values {
        let list = details
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"));
    } else {
        let mut hdr = vec!["Name", "Parent", "Description"];
        let mut props = vec!["name", "parent-name", "description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            props.push("created-at");
            props.push("modified-at");
        }
        let mut table = Table::new("parameter-type");
        table.set_header(&hdr);
        for entry in details {
            let row = entry.get_properties(&props);
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_param_type_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let type_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let rename = subcmd_args.value_of(RENAME_OPT);
    let parent_name = subcmd_args.value_of(PARENT_ARG);
    let mut parent_url: Option<String> = None;
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let details = types.get_details_by_name(rest_cfg, type_name)?;

    if let Some(parent_name) = parent_name {
        if let Some(parent_detail) = types.get_details_by_name(rest_cfg, parent_name)? {
            parent_url = Some(parent_detail.url);
        } else {
            error_message(format!("No parent parameter type '{}' found", parent_name));
            process::exit(46);
        }
    }

    // TODO: parameter type rules

    if let Some(details) = details {
        if description.is_none() && rename.is_none() && parent_name.is_none() {
            warning_message(format!(
                "Parameter type '{}' not updated: nothing to update",
                type_name
            ));
        } else {
            let name = rename.unwrap_or(type_name);
            types.update_type(
                rest_cfg,
                name,
                &details.id,
                description,
                parent_url.as_deref(),
            )?;
            println!("Updated parameter type '{}'", name);
        }
    } else {
        if parent_url.is_none() {
            if let Some(parent_detail) = types.get_details_by_name(rest_cfg, "string")? {
                parent_url = Some(parent_detail.url);
            }
        }
        types.create_type(rest_cfg, type_name, description, parent_url.as_deref())?;
        println!("Created parameter type '{}'", type_name);
    }
    Ok(())
}

fn print_children(level: usize, parent_name: &str, list: &[TypeDetails]) {
    let indent = "  ".repeat(level);
    let mut children: Vec<&TypeDetails> = list
        .iter()
        .filter(|x| x.parent_name == parent_name)
        .collect();
    children.sort_by(|l, r| l.name.cmp(&r.name));
    for child in children {
        // print this child
        println!("{}{}", indent, child.name);

        // recursively go through all of it's children
        print_children(level + 1, &child.name, list);
    }
}

fn proc_param_type_tree(
    _subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let details = types.get_type_details(rest_cfg)?;
    if details.is_empty() {
        println!("No parameter types found.");
    } else {
        for entry in &details {
            if entry.parent_name.is_empty() {
                println!("{}", entry.name);
                print_children(1, &entry.name, &details);
            }
        }
    }
    Ok(())
}

/// Process the 'parameter-types' sub-command
pub fn process_parameter_type_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
) -> Result<()> {
    let types = Types::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_param_type_delete(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_param_type_list(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_param_type_set(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TREE_SUBCMD) {
        proc_param_type_tree(subcmd_args, rest_cfg, &types)?;
    } else {
        warn_missing_subcommand("parameter-types");
    }
    Ok(())
}
