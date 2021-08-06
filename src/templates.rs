use crate::cli::{
    CONFIRM_FLAG, DELETE_SUBCMD, FORMAT_OPT, GET_SUBCMD, LIST_SUBCMD, NAME_ARG, RENAME_OPT,
    SECRETS_FLAG, SET_SUBCMD, TEMPLATE_FILE_OPT, VALUES_FLAG,
};
use crate::database::{OpenApiConfig, Templates};
use crate::table::Table;
use crate::{
    error_message, user_confirm, warn_missing_subcommand, warning_message, ResolvedIds,
    FILE_READ_ERR,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::fs;
use std::process;

/// Process the 'templates' sub-command
pub fn process_templates_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        let proj_name = resolved.project_display_name();
        let proj_id = resolved.project_id();
        let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
        let details = templates.get_details_by_name(rest_cfg, proj_id, template_name)?;

        if let Some(details) = details {
            let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
            if !confirmed {
                confirmed = user_confirm(format!(
                    "Delete template '{}' in project '{}'",
                    template_name, proj_name
                ));
            }

            if !confirmed {
                warning_message(format!(
                    "Template '{}' in project '{}' not deleted!",
                    template_name, proj_name
                ))?;
            } else {
                templates.delete_template(rest_cfg, proj_id, &details.id)?;
                println!(
                    "Deleted template '{}' in project '{}'",
                    template_name, proj_name
                );
            }
        } else {
            warning_message(format!(
                "Template '{}' does not exist for project '{}'!",
                template_name, proj_name
            ))?;
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        let proj_name = resolved.project_display_name();
        let proj_id = resolved.project_id();
        let details = templates.get_template_details(rest_cfg, proj_id)?;
        if details.is_empty() {
            println!("No templates in project '{}'.", proj_name);
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
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let body =
            templates.get_body_by_name(rest_cfg, proj_id, env_id, template_name, show_secrets)?;

        if let Some(body) = body {
            println!("{}", body)
        } else {
            error_message(format!(
                "No template '{}' found in project '{}'.",
                template_name, proj_name
            ))?;
            process::exit(9);
        }
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("preview") {
        let proj_id = resolved.project_id();
        let env_id = resolved.environment_id();
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let filename = subcmd_args.value_of(TEMPLATE_FILE_OPT).unwrap();
        let body = fs::read_to_string(filename).expect(FILE_READ_ERR);
        let result = templates.preview_template(rest_cfg, proj_id, env_id, &body, show_secrets)?;
        println!("{}", result);
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        let filename = subcmd_args.value_of(TEMPLATE_FILE_OPT);
        let proj_id = resolved.project_id();
        let proj_name = resolved.project_display_name();
        let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
        let rename = subcmd_args.value_of(RENAME_OPT);
        let description = subcmd_args.value_of("description");
        let details = templates.get_details_by_name(rest_cfg, proj_id, template_name)?;

        if let Some(details) = details {
            if description.is_none() && rename.is_none() && filename.is_none() {
                warning_message(format!(
                    "Template '{}' not updated: no updated parameters provided",
                    template_name
                ))?;
            } else {
                let name = rename.unwrap_or(&template_name);
                let mut body = None;
                if let Some(filename) = filename {
                    body = Some(fs::read_to_string(filename).expect(FILE_READ_ERR));
                }
                templates.update_template(
                    rest_cfg,
                    proj_id,
                    &details.id,
                    name,
                    description,
                    body.as_deref(),
                )?;
                println!("Updated template '{}' in project '{}'", name, proj_name);
            }
        } else if let Some(filename) = filename {
            let body = fs::read_to_string(filename).expect(FILE_READ_ERR);
            templates.create_template(rest_cfg, proj_id, template_name, &body, description)?;
            println!(
                "Created template '{}' in project '{}'",
                template_name, proj_name
            );
        } else {
            error_message("Must provide a body for a new template".to_owned())?;
            process::exit(8);
        }
    } else {
        warn_missing_subcommand("templates")?;
    }
    Ok(())
}
