use crate::cli::{
    CONFIRM_FLAG, DELETE_SUBCMD, FORMAT_OPT, LIST_SUBCMD, NAME_ARG, RENAME_OPT, SET_SUBCMD,
    VALUES_FLAG,
};
use crate::database::{OpenApiConfig, Projects};
use crate::table::Table;
use crate::{user_confirm, warn_missing_subcommand, warning_message};
use clap::ArgMatches;
use color_eyre::eyre::Result;

fn proc_proj_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let proj_name = subcmd_args.value_of(NAME_ARG).unwrap();
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
    Ok(())
}

fn proc_proj_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
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
    Ok(())
}

fn proc_proj_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    let proj_name = subcmd_args.value_of(NAME_ARG).unwrap();
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
    Ok(())
}

/// Process the 'project' sub-command
pub fn process_project_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    projects: &Projects,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_proj_delete(subcmd_args, rest_cfg, projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_proj_list(subcmd_args, rest_cfg, projects)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_proj_set(subcmd_args, rest_cfg, projects)?;
    } else {
        warn_missing_subcommand("projects")?;
    }
    Ok(())
}
