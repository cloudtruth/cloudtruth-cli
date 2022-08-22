use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, ENV_NAME_ARG, FORMAT_OPT,
    LIST_SUBCMD, NAME_ARG, PARENT_ARG, RENAME_OPT, SET_SUBCMD, SHOW_TIMES_FLAG, TAG_NAME_ARG,
    TAG_SUBCMD, TREE_SUBCMD,
};
use crate::config::DEFAULT_ENV_NAME;
use crate::database::{EnvironmentDetails, Environments, OpenApiConfig};
use crate::lib::{
    current_time, error_message, error_no_environment_message, parse_datetime, user_confirm,
    warn_missing_subcommand, warning_message, DEL_CONFIRM,
};
use crate::table::Table;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use itertools::Itertools;
use std::process;

fn proc_env_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let env_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let details = environments.get_details_by_name(rest_cfg, env_name)?;

    if let Some(details) = details {
        // NOTE: the server is responsible for checking if children exist
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete environment '{}'", env_name), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Environment '{}' not deleted!", env_name));
        } else {
            environments.delete_environment(rest_cfg, details.id)?;
            println!("Deleted environment '{}'", env_name);
        }
    } else {
        warning_message(format!("Environment '{}' does not exist!", env_name));
    }
    Ok(())
}

fn proc_env_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let details = environments.get_environment_details(rest_cfg)?;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    // NOTE: should always have at least the default environment
    if !show_values {
        let list = details.iter().map(|v| v.name.clone()).join("\n");
        println!("{}", list);
    } else {
        let mut table = Table::new("environment");
        let mut hdr = vec!["Name", "Parent", "Description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        table.set_header(&hdr);
        for entry in details {
            let mut row = vec![entry.name, entry.parent_name, entry.description];
            if show_times {
                row.push(entry.created_at);
                row.push(entry.modified_at);
            }
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_env_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let env_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let parent_name = subcmd_args.value_of(PARENT_ARG);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let rename = subcmd_args.value_of(RENAME_OPT);
    let details = environments.get_details_by_name(rest_cfg, env_name)?;

    if let Some(details) = details {
        if parent_name.is_some() && parent_name.unwrap() != details.parent_name.as_str() {
            error_message(format!(
                "Environment '{}' parent cannot be updated.",
                env_name
            ));
            process::exit(6);
        } else if description.is_none() && rename.is_none() {
            warning_message(format!(
                "Environment '{}' not updated: no updated parameters provided",
                env_name
            ));
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
            error_message(format!("No parent environment '{}' found", parent_name));
            process::exit(5);
        }
    }
    Ok(())
}

fn print_children(level: usize, parent_name: &str, list: &[EnvironmentDetails]) {
    let indent = "  ".repeat(level);
    let mut children: Vec<&EnvironmentDetails> = list
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

fn proc_env_tree(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let start = subcmd_args.value_of(NAME_ARG).unwrap();
    let details = environments.get_environment_details(rest_cfg)?;
    if details.iter().filter(|x| x.name == start).last().is_some() {
        println!("{}", start);
        print_children(1, start, &details);
    } else {
        warning_message(format!("No environment '{}' found", start));
    }
    Ok(())
}

fn proc_env_tag_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let env_name = subcmd_args.value_of(ENV_NAME_ARG).unwrap();
    let tag_name = subcmd_args.value_of(TAG_NAME_ARG).unwrap();
    let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);

    let environment_id = environments.get_id(rest_cfg, env_name)?;
    if let Some(env_id) = environment_id {
        if let Some(tag_id) = environments.get_tag_id(rest_cfg, &env_id, tag_name)? {
            // NOTE: the server is responsible for checking if children exist
            if !confirmed {
                confirmed = user_confirm(
                    format!("Delete tag '{}' from environment '{}'", tag_name, env_name),
                    DEL_CONFIRM,
                );
            }

            if !confirmed {
                warning_message(format!(
                    "Tag '{}' in environment '{}' not deleted!",
                    tag_name, env_name
                ));
            } else {
                environments.delete_env_tag(rest_cfg, &env_id, &tag_id)?;
                println!("Deleted tag '{}' from environment '{}'", tag_name, env_name);
            }
        } else {
            warning_message(format!(
                "Environment '{}' does not have a tag '{}'!",
                env_name, tag_name
            ));
        }
    } else {
        warning_message(format!("Environment '{}' does not exist!", env_name));
    }
    Ok(())
}

fn proc_env_tag_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let env_name = subcmd_args.value_of(ENV_NAME_ARG).unwrap();
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let show_usage = subcmd_args.is_present("usage");
    let show_values = show_values(subcmd_args) || show_usage;
    let environment_id = environments.get_id(rest_cfg, env_name)?;

    if let Some(env_id) = environment_id {
        let tags = environments.get_env_tags(rest_cfg, &env_id)?;

        if tags.is_empty() {
            println!("No tags found in environment {}", env_name,);
        } else if !show_values {
            let list = tags.iter().map(|t| t.name.clone()).join("\n");
            println!("{}", list)
        } else {
            let mut table = Table::new("environment-tags");
            let mut hdr = vec!["Name", "Timestamp", "Description"];
            if show_usage {
                hdr.push("Total Reads");
                hdr.push("Last User");
                hdr.push("Last Time");
            }
            table.set_header(&hdr);
            for entry in tags {
                let mut row = vec![entry.name, entry.timestamp, entry.description];
                if show_usage {
                    row.push(entry.total_reads.to_string());
                    row.push(entry.last_use_user);
                    row.push(entry.last_use_time);
                }
                table.add_row(row);
            }
            table.render(fmt)?;
        }
    } else {
        error_no_environment_message(env_name);
        process::exit(14);
    }
    Ok(())
}

fn proc_env_tag_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    let tag_name = subcmd_args.value_of(TAG_NAME_ARG).unwrap();
    let env_name = subcmd_args.value_of(ENV_NAME_ARG).unwrap();
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let rename = subcmd_args.value_of(RENAME_OPT);
    let current = subcmd_args.is_present("current");

    // make sure the user provided something useful for a timestamp
    let time_opt = subcmd_args.value_of("timestamp");
    if time_opt.is_some() && parse_datetime(time_opt).is_none() {
        error_message("Invalid time value -- use an accepted timestamp format".to_string());
        process::exit(16);
    }

    // cannot over-specify
    if time_opt.is_some() && current {
        let msg = "Conflicting arguments: cannot specify both --current and --time.";
        error_message(msg.to_string());
        process::exit(17);
    }

    let timestamp = parse_datetime(time_opt);
    let environment_id = environments.get_id(rest_cfg, env_name)?;
    if let Some(env_id) = environment_id {
        if let Some(tag_id) = environments.get_tag_id(rest_cfg, &env_id, tag_name)? {
            if description.is_none() && timestamp.is_none() && rename.is_none() && !current {
                warning_message(
                    "Nothing changed. Please provide a description, time, or current.".to_string(),
                );
            } else {
                let time_value = if current {
                    Some(current_time())
                } else {
                    timestamp
                };
                let name = rename.unwrap_or(tag_name);
                environments.update_env_tag(
                    rest_cfg,
                    &env_id,
                    &tag_id,
                    name,
                    description,
                    time_value,
                )?;
                println!("Updated tag '{}' in environment '{}'.", name, env_name);
            }
        } else {
            let _ =
                environments.create_env_tag(rest_cfg, &env_id, tag_name, description, timestamp)?;
            println!("Created tag '{}' in environment '{}'.", tag_name, env_name);
        }
    } else {
        error_no_environment_message(env_name);
        process::exit(15);
    }
    Ok(())
}

fn proc_env_tag(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    environments: &Environments,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_env_tag_delete(subcmd_args, rest_cfg, environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_env_tag_list(subcmd_args, rest_cfg, environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_env_tag_set(subcmd_args, rest_cfg, environments)?;
    } else {
        warn_missing_subcommand("environments tag");
    }
    Ok(())
}

/// Process the 'environment' sub-command
pub fn process_environment_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
) -> Result<()> {
    let environments = Environments::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_env_delete(subcmd_args, rest_cfg, &environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_env_list(subcmd_args, rest_cfg, &environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_env_set(subcmd_args, rest_cfg, &environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TREE_SUBCMD) {
        proc_env_tree(subcmd_args, rest_cfg, &environments)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TAG_SUBCMD) {
        proc_env_tag(subcmd_args, rest_cfg, &environments)?;
    } else {
        warn_missing_subcommand("environments");
    }
    Ok(())
}
