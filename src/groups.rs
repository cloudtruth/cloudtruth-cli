use crate::cli::{
    show_values, ADD_USER_OPT, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT,
    GET_SUBCMD, LIST_SUBCMD, NAME_ARG, RENAME_OPT, RM_USER_OPT, SET_SUBCMD, SHOW_TIMES_FLAG,
};
use crate::database::{GroupDetails, Groups, OpenApiConfig, UserError, Users};

use crate::table::Table;
use crate::utils::{
    error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::process;

fn print_group(details: &GroupDetails) {
    printdoc!(
        r#"
            Name: {}
            Description: {}
            ID: {}
            Group URL: {}
            Created At: {}
            Modified At: {}
            Users: {}
        "#,
        details.name,
        details.description,
        details.id,
        details.url,
        details.created_at,
        details.modified_at,
        details.users.join(", ")
    );
}

fn proc_groups_get(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    groups: &Groups,
) -> Result<()> {
    let group_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let response = groups.get_group_details_by_name(rest_cfg, group_name)?;

    if let Some(group) = response {
        print_group(&group);
    } else {
        error_message(format!("The group '{group_name}' could not be found"));
        process::exit(51);
    }
    Ok(())
}

fn proc_groups_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    groups: &Groups,
) -> Result<()> {
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let show_values = show_values(subcmd_args);
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let group_list = groups.get_group_details_list(rest_cfg)?;
    if group_list.is_empty() {
        println!("No groups found!");
    } else if !show_values {
        println!(
            "{}",
            group_list
                .iter()
                .map(|g| g.name.clone())
                .collect::<Vec<String>>()
                .join("\n")
        );
    } else {
        let mut hdr = vec!["Name", "Description"];
        let mut properties = vec!["name", "description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }
        hdr.push("Users");
        properties.push("users");
        let mut table = Table::new("group");
        table.set_header(&hdr);
        for entry in group_list {
            let row = entry.get_properties(&properties);
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_groups_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    groups: &Groups,
) -> Result<()> {
    let group_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let group_id = groups.get_id(rest_cfg, group_name)?;

    if let Some(group_id) = group_id {
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete group '{group_name}'"), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Group '{group_name}' not deleted!"));
        } else {
            groups.delete_group(rest_cfg, &group_id)?;
            println!("Deleted group '{group_name}'");
        }
    } else {
        warning_message(format!("Group '{group_name}' does not exist!"));
    }
    Ok(())
}

fn proc_groups_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    groups: &Groups,
) -> Result<()> {
    let group_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let rename = subcmd_args.value_of(RENAME_OPT);

    /* Look for existing group */
    let found_group = groups.get_group_by_name(rest_cfg, group_name)?;
    /* Update existing group or create new group if not found */
    let group = if let Some(group) = found_group {
        if description.is_some() || rename.is_some() {
            groups.update_group(rest_cfg, &group.id, rename, description)?;
        }
        group
    } else {
        groups.create_group(rest_cfg, group_name, description)?
    };
    /* Convert the provided user names into URLs  */
    let user_name_to_url = |name| {
        let user = Users::new()
            .get_details_by_name(rest_cfg, name)?
            .ok_or_else(|| UserError::UserNotFound(name.to_string()))?;
        Ok(user.user_url)
    };
    let add_user_urls = subcmd_args
        .values_of(ADD_USER_OPT)
        .unwrap_or_default()
        .map(user_name_to_url)
        .collect::<Result<Vec<String>>>()?;
    let rm_user_urls = subcmd_args
        .values_of(RM_USER_OPT)
        .unwrap_or_default()
        .map(user_name_to_url)
        .collect::<Result<Vec<String>>>()?;
    for user_url in add_user_urls {
        groups.add_user_to_group(rest_cfg, &group, &user_url)?;
    }
    for user_url in rm_user_urls {
        groups.remove_user_from_group(rest_cfg, &group, &user_url)?;
    }

    Ok(())
}

/// Process the 'groups' sub-command
pub fn process_groups_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let groups = Groups::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_groups_get(subcmd_args, rest_cfg, &groups)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_groups_list(subcmd_args, rest_cfg, &groups)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_groups_delete(subcmd_args, rest_cfg, &groups)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_groups_set(subcmd_args, rest_cfg, &groups)?;
    } else {
        warn_missing_subcommand("groups");
    }
    Ok(())
}
