use crate::cli::{
    show_values, CHILD_NAMES_OPT, CONFIRM_FLAG, COPY_DEST_NAME_ARG, COPY_SRC_NAME_ARG, COPY_SUBCMD,
    DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, LIST_SUBCMD, NAME_ARG, PARENT_ARG, RECURSIVE_OPT,
    RENAME_OPT, SET_SUBCMD, SHOW_TIMES_FLAG, TREE_SUBCMD,
};
use crate::database::{OpenApiConfig, ProjectDetails, Projects};
use crate::table::Table;
use crate::utils::{
    error_message, parse_key_value_pairs, user_confirm, warn_missing_subcommand, warning_message,
    DEL_CONFIRM,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

fn proc_proj_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let proj_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let proj_id = projects.get_id(rest_cfg, proj_name)?;

    if let Some(proj_id) = proj_id {
        // NOTE: the server is responsible for checking if children exist
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete project '{proj_name}'"), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Project '{proj_name}' not deleted!"));
        } else {
            projects.delete_project(rest_cfg, &proj_id)?;
            println!("Deleted project '{proj_name}'");
        }
    } else {
        warning_message(format!("Project '{proj_name}' does not exist!"));
    }
    Ok(())
}

fn proc_proj_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let details = projects.get_project_details(rest_cfg)?;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    if details.is_empty() {
        println!("No projects found.");
    } else if !show_values {
        let list = details
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"));
    } else {
        let mut table = Table::new("project");
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

fn proc_proj_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let proj_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let rename = subcmd_args.value_of(RENAME_OPT);
    let parent_name = subcmd_args.value_of(PARENT_ARG);
    let mut parent_url: Option<String> = None;
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let details = projects.get_details_by_name(rest_cfg, proj_name, false)?;

    if let Some(parent_name) = parent_name {
        if parent_name.is_empty() {
            parent_url = Some("".to_string());
        } else if let Some(parent_detail) =
            projects.get_details_by_name(rest_cfg, parent_name, false)?
        {
            parent_url = Some(parent_detail.url);
        } else {
            error_message(format!("No parent project '{parent_name}' found"));
            process::exit(19);
        }
    }

    if let Some(details) = details {
        if description.is_none() && rename.is_none() && parent_name.is_none() {
            warning_message(format!(
                "Project '{proj_name}' not updated: no updated parameters provided"
            ));
        } else {
            let name = rename.unwrap_or(proj_name);
            projects.update_project(
                rest_cfg,
                name,
                &details.id,
                description,
                parent_url.as_deref(),
                None,
            )?;
            println!("Updated project '{name}'");
        }
    } else {
        projects.create_project(
            rest_cfg,
            proj_name,
            description,
            parent_url.as_deref(),
            None,
        )?;
        println!("Created project '{proj_name}'");
    }
    Ok(())
}

fn print_children(level: usize, parent_name: &str, list: &[ProjectDetails]) {
    let indent = "  ".repeat(level);
    let mut children: Vec<&ProjectDetails> = list
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

fn proc_proj_tree(
    _subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let details = projects.get_project_details(rest_cfg)?;
    if details.is_empty() {
        println!("No projects found.");
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

fn proc_proj_copy(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let src_proj_name = subcmd_args.value_of(COPY_SRC_NAME_ARG).unwrap();
    let dest_proj_name = subcmd_args.value_of(COPY_DEST_NAME_ARG).unwrap();
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let recursive = subcmd_args.is_present(RECURSIVE_OPT);
    let child_names = subcmd_args.value_of(CHILD_NAMES_OPT);
    if !recursive && child_names.is_some() {
        error_message("--recursive option is required when using --child-names");
        process::exit(60);
    }
    let child_names = child_names.map(|child_names| {
        parse_key_value_pairs(child_names).unwrap_or_else(|| {
            error_message(format!("Unable to parse key/value pairs: {child_names}"));
            process::exit(61);
        })
    });
    if let Some(src_proj) = projects.get_details_by_name(rest_cfg, src_proj_name, false)? {
        projects.copy_project(
            rest_cfg,
            &src_proj.id,
            dest_proj_name,
            description,
            recursive,
            child_names,
        )?;
        println!("Copied project '{src_proj_name}' to '{dest_proj_name}'.");
    } else {
        warning_message(format!("No project '{src_proj_name}' found"));
    }
    Ok(())
}

/// Process the 'project' sub-command
pub fn process_project_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let projects = Projects::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_proj_delete(subcmd_args, rest_cfg, &projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_proj_list(subcmd_args, rest_cfg, &projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_proj_set(subcmd_args, rest_cfg, &projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TREE_SUBCMD) {
        proc_proj_tree(subcmd_args, rest_cfg, &projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(COPY_SUBCMD) {
        proc_proj_copy(subcmd_args, rest_cfg, &projects)?;
    } else {
        warn_missing_subcommand("projects");
    }
    Ok(())
}
