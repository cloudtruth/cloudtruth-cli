use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, GET_SUBCMD,
    INTEGRATION_NAME_ARG, LIST_SUBCMD, PUSH_NAME_ARG, PUSH_SUBCMD, RENAME_OPT, SET_SUBCMD,
    SHOW_TIMES_FLAG, TASKS_SUBCMD,
};
use crate::database::{
    last_from_url, Environments, IntegrationError, Integrations, OpenApiConfig, ProjectDetails,
    Projects, PushDetails,
};
use crate::integrations::integration_not_found_message;
use crate::table::Table;
use crate::{
    error_message, help_message, user_confirm, warn_missing_subcommand, warning_message,
    DEL_CONFIRM,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::collections::{HashMap, HashSet};
use std::process;

fn push_not_found_message(push_name: &str, integ_name: Option<&str>) -> String {
    if let Some(integ_name) = integ_name {
        format!(
            "Push action '{}' not found in integration '{}'",
            push_name, integ_name
        )
    } else {
        format!("Push action '{}' not found", push_name)
    }
}

fn env_url_from_tag_url(tag_url: &str) -> &str {
    // NOTE: must keep trailing slash on original to equal what comes from EnvironmentDetails.url
    let parts: Vec<&str> = tag_url.split("tags/").collect();
    parts[0]
}

fn env_id_from_tag_url(tag_url: &str) -> &str {
    last_from_url(env_url_from_tag_url(tag_url))
}

fn resolve_tag_names(rest_cfg: &OpenApiConfig, pushes: &mut [PushDetails]) {
    // if there are no pushes with tag URLs, we're done
    if !pushes.iter().any(|x| !x.tag_urls.is_empty()) {
        return;
    }

    let environments = Environments::new();
    let env_map = environments.get_url_name_map(rest_cfg);

    // get a list of environments for which we need tags
    let mut env_list: HashSet<String> = HashSet::new();
    for push in pushes.iter() {
        for tag_url in &push.tag_urls {
            let env_id = env_id_from_tag_url(tag_url).to_string();
            env_list.insert(env_id);
        }
    }

    // create a map of tag_url => "env_name:tag_name"
    let mut tag_map: HashMap<String, String> = HashMap::new();
    for env_id in env_list {
        let env_tags = environments
            .get_env_tags(rest_cfg, &env_id)
            .unwrap_or_default();
        for tag in env_tags {
            let env_url = env_url_from_tag_url(&tag.url);
            let env_name = env_map.get(env_url).unwrap_or(&env_id).clone();
            let tag_name = format!("{}:{}", env_name, tag.name);
            tag_map.insert(tag.url.clone(), tag_name);
        }
    }

    // now that we have all the info, put it back into the pushes
    for push in pushes {
        for tag_url in &push.tag_urls {
            let tag_name = tag_map.get(tag_url).unwrap_or(tag_url).clone();
            push.tag_names.push(tag_name);
        }
    }
}

fn resolve_project_names(rest_cfg: &OpenApiConfig, pushes: &mut [PushDetails]) {
    // if there are no pushes with tag URLs, we're done
    if !pushes.iter().any(|x| !x.project_urls.is_empty()) {
        return;
    }

    let projects = Projects::new();
    let proj_map = projects.get_url_name_map(rest_cfg);
    let default_proj_name = "Unknown".to_string();

    for entry in pushes.iter_mut() {
        for proj_url in entry.project_urls.iter() {
            let proj_name = proj_map
                .get(proj_url.as_str())
                .unwrap_or(&default_proj_name);
            entry.project_names.push(proj_name.clone());
        }
    }
}

fn project_names_to_urls(proj_names: &[&str], proj_details: &[ProjectDetails]) -> Vec<String> {
    let mut proj_urls: Vec<String> = vec![];
    for name in proj_names {
        let mut found = false;
        for details in proj_details {
            if details.name.as_str() == *name {
                found = true;
                proj_urls.push(details.url.clone());
                break;
            }
        }
        if !found {
            error_message(format!("Project '{}' not found", name));
            process::exit(36);
        }
    }
    proj_urls
}

fn get_tag_name_to_url_map(
    rest_cfg: &OpenApiConfig,
    tag_names: &[&str],
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();

    // create a de-duplicated set of environments for which we need to grab the tags
    let mut env_names: HashSet<String> = HashSet::new();
    for full_tag in tag_names {
        let parts: Vec<&str> = full_tag.split(':').collect();
        env_names.insert(parts[0].to_string());
    }

    // loop through all the environments, and build up our tag map
    let environments = Environments::new();
    let env_details = environments
        .get_environment_details(rest_cfg)
        .unwrap_or_default();
    for env_name in env_names {
        let found = env_details
            .iter()
            .find(|d| d.name == env_name)
            .map(|d| d.id.clone());
        if let Some(env_id) = found {
            let tag_details = environments
                .get_env_tags(rest_cfg, &env_id)
                .unwrap_or_default();
            for t in tag_details {
                let full_name = format!("{}:{}", env_name, &t.name);
                result.insert(full_name, t.url.clone());
            }
        } else {
            error_message(format!("Environment '{}' not found", env_name));
            process::exit(37);
        }
    }

    result
}

fn tag_names_to_urls(tag_names: &[&str], tag_map: &HashMap<String, String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for full_tag in tag_names {
        let map_value: Option<&String> = tag_map.get(&full_tag.to_string());
        if let Some(url) = map_value {
            result.push(url.clone());
        } else {
            error_message(format!("Did not find tag for {}", full_tag));
            process::exit(38);
        }
    }
    result
}

fn resolve_push_details(
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
    integ_name: Option<&str>,
    push_name: &str,
) -> Result<Option<PushDetails>, IntegrationError> {
    if let Some(integ_name) = integ_name {
        let integ_resp = integrations.get_id(rest_cfg, integ_name)?;
        if let Some(integ_id) = integ_resp {
            let push_resp = integrations.get_push_by_name(rest_cfg, &integ_id, push_name)?;
            if let Some(details) = push_resp {
                let mut result = details;
                result.integration_name = integ_name.to_string();
                Ok(Some(result))
            } else {
                Ok(None)
            }
        } else {
            error_message(integration_not_found_message(integ_name));
            process::exit(40);
        }
    } else {
        let named_details = integrations.get_all_pushes_by_name(rest_cfg, push_name)?;

        match named_details.len() {
            0 => Ok(None),
            1 => Ok(Some(named_details[0].clone())),
            _ => {
                let integration_names: Vec<String> = named_details
                    .iter()
                    .map(|d| d.integration_name.clone())
                    .collect();
                error_message(format!(
                    "Found '{}' in integrations: {}",
                    push_name,
                    integration_names.join(", ")
                ));
                help_message(
                    "Use the --integration option to specify a specific integration.".to_string(),
                );
                process::exit(41);
            }
        }
    }
}

fn proc_action_push_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let resolved = resolve_push_details(rest_cfg, integrations, integ_name, push_name)?;

    if let Some(details) = resolved {
        // NOTE: the server is responsible for checking if children exist
        let integ_name = details.integration_name.clone();
        let integ_id = details.get_integration_id();
        let push_id = details.id.clone();
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            let msg = format!(
                "Delete push '{}' from integration '{}'",
                push_name, integ_name
            );
            confirmed = user_confirm(msg, DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Push '{}' not deleted from !", push_name));
        } else {
            integrations.delete_push(rest_cfg, &integ_id, &push_id)?;
            println!("Deleted push '{}' from '{}'", details.name, integ_name);
        }
    } else {
        warning_message(push_not_found_message(push_name, integ_name));
    }
    Ok(())
}

fn print_push_details(push: &PushDetails) {
    let error_info = if push.last_task.state != "success" {
        format!(
            "{}: {}",
            push.last_task.error_code, push.last_task.error_detail
        )
    } else {
        "".to_string()
    };

    printdoc!(
        r#"
        Name: {}
        Provider: {}
        Integration: {}
        Service: {}
        Region: {}
        Resource: {}
        Description: {}
        Projects: {}
        Tags: {}
        ID: {}
        URL: {}
        Project URLs: {}
        Tag URLs: {}
        Created At: {}
        Modified At: {}
        Last task:
          Reason: {}
          State: {}
          ID: {}
          URL: {}
          Error Info: {}
          Created At: {}
          Modified At: {}
        "#,
        push.name,
        push.provider,
        push.integration_name,
        push.service,
        push.region,
        push.resource,
        push.description,
        push.project_names.join(", "),
        push.tag_names.join(", "),
        push.id,
        push.url,
        push.project_urls.join(", "),
        push.tag_urls.join(", "),
        push.created_at,
        push.modified_at,
        push.last_task.reason,
        push.last_task.state,
        push.last_task.id,
        push.last_task.url,
        error_info,
        push.last_task.created_at,
        push.last_task.modified_at,
    );
}

fn proc_action_push_get(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let resolved = resolve_push_details(rest_cfg, integrations, integ_name, push_name)?;

    if let Some(details) = resolved {
        // put this into a list, so we can resolve with larger functions
        let mut pushes = vec![details];
        resolve_project_names(rest_cfg, &mut pushes);
        resolve_tag_names(rest_cfg, &mut pushes);
        print_push_details(&pushes[0]);
    } else {
        error_message(push_not_found_message(push_name, integ_name));
        process::exit(31);
    }
    Ok(())
}

fn proc_action_push_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let qualifier: String;
    let show_integration: bool;
    let mut pushes: Vec<PushDetails>;

    if let Some(integ_name) = integ_name {
        qualifier = format!(" for integration '{}'", integ_name);
        show_integration = false;
        if let Some(integ_id) = integrations.get_id(rest_cfg, integ_name)? {
            pushes = integrations.get_push_list(rest_cfg, &integ_id)?;
        } else {
            error_message(integration_not_found_message(integ_name));
            process::exit(30);
        }
    } else {
        qualifier = "".to_string();
        show_integration = true;
        pushes = integrations.get_all_pushes(rest_cfg)?;
    }

    if pushes.is_empty() {
        println!("No pushes found{}", qualifier);
    } else if !show_values {
        let list = pushes
            .iter()
            .map(|d| d.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut hdr = vec![
            "Name",
            "Projects",
            "Tags",
            "Service",
            "Status",
            "Last Push Time",
        ];
        let mut properties = vec![
            "name",
            "project-names",
            "tag-names",
            "service",
            "task-state",
            "task-time",
        ];

        resolve_project_names(rest_cfg, &mut pushes);
        resolve_tag_names(rest_cfg, &mut pushes);

        if show_integration {
            hdr.insert(1, "Integration");
            properties.insert(1, "integration");
        }
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }

        let mut table = Table::new("action-push");
        table.set_header(&hdr);
        for entry in pushes {
            table.add_row(entry.get_properties(&properties));
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_action_push_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let updated_name = subcmd_args.value_of(RENAME_OPT).unwrap_or(push_name);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let resource = subcmd_args.value_of("resource");
    let region = subcmd_args.value_of("region").unwrap();
    let service = subcmd_args.value_of("service").unwrap();
    let proj_to_add: Vec<&str> = subcmd_args
        .values_of("project-add")
        .unwrap_or_default()
        .collect();
    let proj_to_sub: Vec<&str> = subcmd_args
        .values_of("project-sub")
        .unwrap_or_default()
        .collect();
    let mut proj_add_ids = vec![];
    let mut proj_sub_ids = vec![];
    let tag_to_add: Vec<&str> = subcmd_args
        .values_of("tag-add")
        .unwrap_or_default()
        .collect();
    let tag_to_sub: Vec<&str> = subcmd_args
        .values_of("tag-sub")
        .unwrap_or_default()
        .collect();
    let mut tag_add_ids = vec![];
    let mut tag_sub_ids = vec![];

    if !proj_to_add.is_empty() || !proj_to_sub.is_empty() {
        let projects = Projects::new();
        let proj_details = projects.get_project_details(rest_cfg)?;
        proj_add_ids = project_names_to_urls(&proj_to_add, &proj_details);
        proj_sub_ids = project_names_to_urls(&proj_to_sub, &proj_details);
    }

    if !tag_to_add.is_empty() || !tag_to_sub.is_empty() {
        let mut all_tags: Vec<&str> = tag_to_add.clone();
        all_tags.append(tag_to_sub.clone().as_mut());

        let tag_map = get_tag_name_to_url_map(rest_cfg, &all_tags);
        tag_add_ids = tag_names_to_urls(&tag_to_add, &tag_map);
        tag_sub_ids = tag_names_to_urls(&tag_to_sub, &tag_map);
    }

    let resolved = resolve_push_details(rest_cfg, integrations, integ_name, push_name)?;
    if let Some(details) = resolved {
        // update code
        if subcmd_args.occurrences_of("region") > 0 {
            warning_message(format!(
                "The --region is ignored for updates to '{}",
                push_name
            ));
        }
        if subcmd_args.occurrences_of("service") > 0 {
            warning_message(format!(
                "The --service is ignored for updates to '{}",
                push_name
            ));
        }

        let updated_resource = resource.unwrap_or(&details.resource);
        let mut project_ids = details.project_urls.clone();
        project_ids.append(&mut proj_add_ids);
        project_ids.retain(|i| !proj_sub_ids.contains(i));
        let mut tag_ids = details.tag_urls.clone();
        tag_ids.append(&mut tag_add_ids);
        tag_ids.retain(|i| !tag_sub_ids.contains(i));
        integrations.update_push(
            rest_cfg,
            &details.get_integration_id(),
            &details.id,
            updated_name,
            updated_resource,
            description,
            project_ids,
            tag_ids,
        )?;
        println!(
            "Updated push '{}' in integration '{}'",
            updated_name, details.integration_name
        );
    } else if let Some(integ_name) = integ_name {
        let response_integ = integrations.get_id(rest_cfg, integ_name)?;
        if let Some(integ_id) = response_integ {
            integrations.create_push(
                rest_cfg,
                &integ_id,
                push_name,
                resource.unwrap_or("/{{ environment }}/{{ project }}/{{ parameter }}"),
                region,
                service,
                description,
                proj_add_ids.iter().map(String::from).collect(),
                tag_add_ids.iter().map(String::from).collect(),
            )?;
            println!(
                "Created push '{}' in integration '{}'",
                push_name, integ_name
            );
        } else {
            error_message(integration_not_found_message(integ_name));
            process::exit(30);
        }
    } else {
        error_message("Must specify an integration on create!".to_string());
        process::exit(42);
    }
    Ok(())
}

fn proc_action_push_sync(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let resolved = resolve_push_details(rest_cfg, integrations, integ_name, push_name)?;

    if let Some(details) = resolved {
        integrations.sync_push(rest_cfg, &details)?;
        println!(
            "Synchronized push '{}' for integration '{}'",
            push_name, details.integration_name
        );
    } else {
        error_message(push_not_found_message(push_name, integ_name));
        process::exit(31);
    }
    Ok(())
}

fn proc_action_push_tasks(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    let integ_name = subcmd_args.value_of(INTEGRATION_NAME_ARG);
    let push_name = subcmd_args.value_of(PUSH_NAME_ARG).unwrap();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let resolved = resolve_push_details(rest_cfg, integrations, integ_name, push_name)?;

    if let Some(details) = resolved {
        let push_id = details.id.clone();
        let integ_id = details.get_integration_id();
        let integ_name = details.integration_name;
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

            let mut table = Table::new("action-push-task");
            table.set_header(&hdr);
            for entry in tasks {
                table.add_row(entry.get_properties(&properties));
            }
            table.render(fmt)?;
        }
    } else {
        error_message(push_not_found_message(push_name, integ_name));
        process::exit(31);
    }
    Ok(())
}

fn proc_action_push_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_action_push_delete(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_action_push_get(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_action_push_list(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_action_push_set(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("sync") {
        proc_action_push_sync(subcmd_args, rest_cfg, integrations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TASKS_SUBCMD) {
        proc_action_push_tasks(subcmd_args, rest_cfg, integrations)?;
    } else {
        warn_missing_subcommand("actions pushes");
    }
    Ok(())
}

/// Process the 'integrations' sub-command
pub fn process_actions_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    integrations: &Integrations,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(PUSH_SUBCMD) {
        proc_action_push_command(subcmd_args, rest_cfg, integrations)?;
    } else {
        warn_missing_subcommand("actions");
    }
    Ok(())
}
