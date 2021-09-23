use crate::cli::{
    AS_OF_ARG, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, EDIT_SUBCMD, FORMAT_OPT, GET_SUBCMD,
    HISTORY_SUBCMD, LIST_SUBCMD, NAME_ARG, RENAME_OPT, SECRETS_FLAG, SET_SUBCMD, SHOW_TIMES_FLAG,
    TEMPLATE_FILE_OPT, VALUES_FLAG,
};
use crate::database::{HistoryAction, OpenApiConfig, TemplateHistory, Templates};
use crate::table::Table;
use crate::{
    error_message, parse_datetime, parse_tag, user_confirm, warn_missing_subcommand,
    warning_message, ResolvedIds, DEL_CONFIRM, FILE_READ_ERR,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::fs;
use std::process;

const TEMPLATE_HISTORY_PROPERTIES: &[&str] = &["name", "description", "body"];

fn proc_template_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let details = templates.get_details_by_name(rest_cfg, proj_id, template_name)?;

    if let Some(details) = details {
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(
                format!(
                    "Delete template '{}' in project '{}'",
                    template_name, proj_name
                ),
                DEL_CONFIRM,
            );
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
    Ok(())
}

fn proc_template_edit(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let result = templates.get_unevaluated_details(rest_cfg, proj_id, template_name)?;

    if let Some(details) = result {
        let new_body = edit::edit(details.body.as_bytes())?;
        if new_body != details.body {
            templates.update_template(
                rest_cfg,
                proj_id,
                &details.id,
                template_name,
                None,
                Some(&new_body),
            )?;
            println!(
                "Updated template '{}' in project '{}'",
                template_name, proj_name
            );
        } else {
            println!("Nothing to update in template '{}'", template_name);
        }
    } else {
        println!(
            "Template '{}' does not exist for project '{}'",
            template_name, proj_name
        );
    }
    Ok(())
}

fn proc_template_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = subcmd_args.is_present(VALUES_FLAG) || show_times;
    let details = templates.get_template_details(rest_cfg, proj_id)?;
    if details.is_empty() {
        println!("No templates in project '{}'.", proj_name);
    } else if !show_values {
        let list = details
            .iter()
            .map(|n| n.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut table = Table::new("template");
        let mut hdr = vec!["Name", "Description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        table.set_header(&hdr);
        for entry in details {
            let mut row = vec![entry.name, entry.description];
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

fn proc_template_get(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let raw = subcmd_args.is_present("raw");

    let body = if raw {
        let result = templates.get_unevaluated_details(rest_cfg, proj_id, template_name)?;
        match result {
            Some(details) => Some(details.body),
            _ => None,
        }
    } else {
        templates.get_body_by_name(rest_cfg, proj_id, env_id, template_name, show_secrets)?
    };

    if let Some(body) = body {
        println!("{}", body)
    } else {
        error_message(format!(
            "No template '{}' found in project '{}'.",
            template_name, proj_name
        ))?;
        process::exit(9);
    }
    Ok(())
}

fn proc_template_preview(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let filename = subcmd_args.value_of(TEMPLATE_FILE_OPT).unwrap();
    let body = fs::read_to_string(filename).expect(FILE_READ_ERR);
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let result =
        templates.preview_template(rest_cfg, proj_id, env_id, &body, show_secrets, as_of, tag)?;
    println!("{}", result);
    Ok(())
}

fn proc_template_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let filename = subcmd_args.value_of(TEMPLATE_FILE_OPT);
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let rename = subcmd_args.value_of(RENAME_OPT);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let details = templates.get_details_by_name(rest_cfg, proj_id, template_name)?;

    if let Some(details) = details {
        if description.is_none() && rename.is_none() && filename.is_none() {
            warning_message(format!(
                "Template '{}' not updated: no updated parameters provided",
                template_name
            ))?;
        } else {
            let name = rename.unwrap_or(template_name);
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
    Ok(())
}

/// Looks for the earlier time than this... It relies on the reverse time order.
fn find_previous(
    history: &[TemplateHistory],
    current: &TemplateHistory,
) -> Option<TemplateHistory> {
    let mut found = None;
    let curr_id = current.get_id();
    let curr_date = current.get_date();
    for entry in history {
        if entry.get_id() == curr_id && entry.get_date() < curr_date {
            found = Some(entry.clone());
            break;
        }
    }
    found
}

fn get_changes(
    current: &TemplateHistory,
    previous: Option<TemplateHistory>,
    properties: &[&str],
) -> Vec<String> {
    let mut changes = vec![];
    if let Some(prev) = previous {
        if current.get_action() != HistoryAction::Delete {
            for property in properties {
                let curr_value = current.get_property(property);
                if prev.get_property(property) != curr_value {
                    changes.push(format!("{}: {}", property, curr_value))
                }
            }
        }
    } else {
        // NOTE: print this info even on a delete, if there's nothing earlier
        for property in properties {
            let curr_value = current.get_property(property);
            if !curr_value.is_empty() {
                changes.push(format!("{}: {}", property, curr_value))
            }
        }
    }
    changes
}

fn proc_template_history(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let template_name = subcmd_args.value_of(NAME_ARG);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let modifier;
    let add_name;
    let history: Vec<TemplateHistory>;

    if let Some(temp_name) = template_name {
        let template_id;
        modifier = format!("for '{}' ", temp_name);
        add_name = false;
        if let Some(details) = templates.get_details_by_name(rest_cfg, proj_id, temp_name)? {
            template_id = details.id;
        } else {
            error_message(format!(
                "Did not find '{}' in project '{}'",
                temp_name, proj_name
            ))?;
            process::exit(13);
        }
        history = templates.get_history_for(rest_cfg, proj_id, &template_id, as_of, tag)?;
    } else {
        modifier = "".to_string();
        add_name = true;
        history = templates.get_histories(rest_cfg, proj_id, as_of, tag)?;
    };

    if history.is_empty() {
        println!(
            "No template history {}in project '{}'.",
            modifier, proj_name
        );
    } else {
        let name_index = 2;
        let mut table = Table::new("template-history");
        let mut hdr: Vec<&str> = vec!["Date", "Action", "Changes"];
        if add_name {
            hdr.insert(name_index, "Name");
        }
        table.set_header(&hdr);

        let orig_list = history.clone();
        for ref entry in history {
            let prev = find_previous(&orig_list, entry);
            let changes = get_changes(entry, prev, TEMPLATE_HISTORY_PROPERTIES);
            let mut row = vec![
                entry.date.clone(),
                entry.change_type.to_string(),
                changes.join("\n"),
            ];
            if add_name {
                row.insert(name_index, entry.name.clone())
            }
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_template_validate(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let template_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let show_secrets = true; // make sure we're completely evaluating

    let response =
        templates.get_body_by_name(rest_cfg, proj_id, env_id, template_name, show_secrets)?;
    if response.is_some() {
        println!("Success");
    } else {
        error_message(format!(
            "No template '{}' found in project '{}'.",
            template_name, proj_name
        ))?;
        process::exit(9);
    }
    Ok(())
}

/// Process the 'templates' sub-command
pub fn process_templates_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    templates: &Templates,
    resolved: &ResolvedIds,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_template_delete(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(EDIT_SUBCMD) {
        proc_template_edit(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_template_list(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_template_get(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("preview") {
        proc_template_preview(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_template_set(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(HISTORY_SUBCMD) {
        proc_template_history(subcmd_args, rest_cfg, templates, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("validate") {
        proc_template_validate(subcmd_args, rest_cfg, templates, resolved)?;
    } else {
        warn_missing_subcommand("templates")?;
    }
    Ok(())
}
