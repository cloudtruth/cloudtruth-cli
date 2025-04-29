use crate::binary_name;
use crate::cli::{
    show_values, true_false_option, AS_OF_ARG, CONFIRM_FLAG, COPY_DEST_NAME_ARG, COPY_SRC_NAME_ARG,
    COPY_SUBCMD, DELETE_SUBCMD, DESCRIPTION_OPT, DIFF_SUBCMD, FORMAT_OPT, GET_SUBCMD,
    IMMEDIATE_PARAMETERS_FLAG, JMES_PATH_ARG, KEY_ARG, LIST_SUBCMD, PUSH_SUBCMD, RENAME_OPT,
    RULE_MAX_ARG, RULE_MAX_LEN_ARG, RULE_MIN_ARG, RULE_MIN_LEN_ARG, RULE_NO_MAX_ARG,
    RULE_NO_MAX_LEN_ARG, RULE_NO_MIN_ARG, RULE_NO_MIN_LEN_ARG, RULE_NO_REGEX_ARG, RULE_REGEX_ARG,
    SECRETS_FLAG, SET_SUBCMD, SHOW_TIMES_FLAG,
};
use crate::database::{
    EnvironmentDetails, Environments, OpenApiConfig, ParamExportFormat, ParamExportOptions,
    ParamRuleType, ParameterDetails, ParameterError, Parameters, Projects, ResolvedDetails,
    TaskStepDetails,
};
use crate::subprocess::EnvSettings;
use crate::table::Table;
use crate::utils::{
    error_message, format_param_error, help_message, parse_datetime, parse_tag, user_confirm,
    warn_missing_subcommand, warn_unresolved_params, warning_message, DEL_CONFIRM, FILE_READ_ERR,
    REDACTED,
};
use clap::ArgMatches;
use cloudtruth_config::DEFAULT_ENV_NAME;
use color_eyre::eyre::Result;
use color_eyre::Report;
use indoc::printdoc;
use rpassword::read_password;
use std::env;
use std::fs;
use std::process;
use std::str::FromStr;

fn proc_param_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let mut param_id = None;
    let mask_secrets = true; // no need to fetch secrets
    let immediate_parameters = false;
    let evaluate = false;
    let response = parameters.get_details_by_name(
        rest_cfg,
        proj_id,
        env_id,
        key_name,
        evaluate,
        immediate_parameters,
        mask_secrets,
        None,
        None,
    )?;
    if let Some(details) = response {
        if !details.project_url.contains(proj_id) {
            error_message(format!(
                "Parameter '{}' must be deleted from project '{}' -- it is not part of project '{}'",
                key_name, details.project_name, proj_name
            ));
            process::exit(24);
        }
        param_id = Some(details.id);
    }
    if param_id.is_none() {
        println!(
            "Did not find parameter '{}' to delete from project '{}'.",
            key_name,
            resolved.project_display_name(),
        );
        return Ok(());
    }

    if !confirmed {
        printdoc!(
            r#"

                Deleting a parameter removes it from the project for all environments.
                You can use '{} parameter unset' to delete the value from
                the current environment.

            "#,
            binary_name(),
        );
        if !user_confirm(
            format!("Delete parameter '{key_name}' from project '{proj_name}'"),
            DEL_CONFIRM,
        ) {
            return Ok(());
        }
    }

    let result =
        parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id.unwrap().as_str())?;
    match result {
        Some(_) => {
            println!(
                "Removed parameter '{}' from project '{}'.",
                key_name,
                resolved.project_display_name(),
            );
        }
        _ => {
            println!(
                "Failed to remove parameter '{}' from project '{}'.",
                key_name,
                resolved.project_display_name(),
            );
        }
    };
    Ok(())
}

fn proc_param_copy(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<(), ParameterError> {
    let src_param_name = subcmd_args.value_of(COPY_SRC_NAME_ARG).unwrap();
    let dest_param_name = subcmd_args.value_of(COPY_DEST_NAME_ARG).unwrap();
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    if let Some(src_param) = parameters.get_details_by_name(
        rest_cfg,
        proj_id,
        env_id,
        src_param_name,
        false,
        false,
        true,
        None,
        None,
    )? {
        parameters.copy_param(
            rest_cfg,
            proj_id,
            &src_param.id,
            dest_param_name,
            description,
            &src_param.project_url,
        )?;
        println!(
            "Copied parameter '{src_param_name}' to '{dest_param_name}' in project '{proj_name}'."
        );
    } else {
        warning_message(format!(
            "No parameter '{src_param_name}' found in project '{proj_name}'."
        ));
    }
    Ok(())
}

fn proc_param_diff(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let immediate_parameters = subcmd_args.is_present(IMMEDIATE_PARAMETERS_FLAG);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let properties: Vec<&str> = subcmd_args.values_of("properties").unwrap().collect();
    let as_list: Vec<&str> = subcmd_args
        .values_of(AS_OF_ARG)
        .unwrap_or_default()
        .collect();
    let env_list: Vec<&str> = subcmd_args.values_of("ENV").unwrap_or_default().collect();
    let max_len: usize = 2;

    if env_list.len() > max_len {
        warning_message(format!(
            "Can specify a maximum of {max_len} environment values."
        ));
        return Ok(());
    }
    if as_list.len() > max_len {
        warning_message(format!("Can specify a maximum of {max_len} as-of values."));
        return Ok(());
    }

    let env1_name: String;
    let env2_name: String;
    if env_list.len() == 2 {
        env1_name = env_list[0].to_string();
        env2_name = env_list[1].to_string();
    } else if env_list.len() == 1 {
        env1_name = resolved.environment_display_name().to_string();
        env2_name = env_list[0].to_string();
    } else {
        env1_name = resolved.environment_display_name().to_string();
        env2_name = resolved.environment_display_name().to_string();
    }

    let as_tag1: Option<&str>;
    let as_tag2: Option<&str>;
    if as_list.len() == 2 {
        as_tag1 = Some(as_list[0]);
        as_tag2 = Some(as_list[1]);
    } else if as_list.len() == 1 {
        // puts the specified time in other column
        as_tag1 = None;
        as_tag2 = Some(as_list[0]);
    } else {
        as_tag1 = None;
        as_tag2 = None;
    }

    let as_of1 = parse_datetime(as_tag1);
    let as_of2 = parse_datetime(as_tag2);
    let tag1 = parse_tag(as_tag1);
    let tag2 = parse_tag(as_tag2);

    if env1_name == env2_name && as_tag1 == as_tag2 {
        warning_message("Invalid comparing an environment to itself".to_string());
        return Ok(());
    }

    let header1: String;
    let header2: String;
    if env1_name == env2_name {
        header1 = as_tag1.unwrap_or("Current").to_string();
        header2 = as_tag2.unwrap_or("Unspecified").to_string();
    } else if as_tag1 == as_tag2 {
        header1 = env1_name.to_string();
        header2 = env2_name.to_string();
    } else {
        header1 = match as_tag1 {
            Some(a) => format!("{env1_name} ({a})"),
            _ => env1_name.to_string(),
        };
        header2 = match as_tag2 {
            Some(a) => format!("{env2_name} ({a})"),
            _ => env2_name.to_string(),
        };
    }

    // fetch all environments once, and then determine id's from the same map that is
    // used to resolve the environment names.
    let environments = Environments::new();
    let env_url_map = environments.get_url_name_map(rest_cfg);
    let env1_id = environments.id_from_map(&env1_name, &env_url_map)?;
    let env2_id = environments.id_from_map(&env2_name, &env_url_map)?;

    let proj_id = resolved.project_id();
    let env1_values = parameters.get_parameter_detail_map(
        rest_cfg,
        proj_id,
        &env1_id,
        immediate_parameters,
        !show_secrets,
        as_of1,
        tag1,
    )?;
    let env2_values = parameters.get_parameter_detail_map(
        rest_cfg,
        proj_id,
        &env2_id,
        immediate_parameters,
        !show_secrets,
        as_of2,
        tag2,
    )?;

    // get the names from both lists to make sure we get the added/deleted parameters, too
    let mut param_list: Vec<String> = env1_values.keys().cloned().collect();
    param_list.append(&mut env2_values.keys().cloned().collect());
    param_list.sort_by_key(|l| l.to_lowercase());
    param_list.dedup();

    let default_param = ParameterDetails::default();
    let mut added = false;
    let mut table = Table::new("parameter");
    let mut errors: Vec<String> = vec![];
    table.set_header(&["Parameter", &header1, &header2]);
    for param_name in param_list {
        let details1 = env1_values.get(&param_name).unwrap_or(&default_param);
        let details2 = env2_values.get(&param_name).unwrap_or(&default_param);
        let env1 = details1.get_properties(&properties).join(",\n");
        let env2 = details2.get_properties(&properties).join(",\n");
        if !details1.error.is_empty() {
            errors.push(format_param_error(&param_name, &details1.error))
        }
        // NOTE: do not put redundant errors on the list, but the errors could be due to
        //       different FQNs
        if !details2.error.is_empty() && details1.error != details2.error {
            errors.push(format_param_error(&param_name, &details2.error))
        }
        if env1 != env2 {
            table.add_row(vec![param_name, env1, env2]);
            added = true;
        }
    }
    if added {
        table.render(fmt)?;
    } else {
        println!("No parameters or differences in compared properties found.");
    }
    warn_unresolved_params(&errors);
    Ok(())
}

fn get_env_order_for(parent_name: &str, environments: &[EnvironmentDetails]) -> Vec<String> {
    let mut result = vec![];
    let mut children: Vec<&EnvironmentDetails> = environments
        .iter()
        .filter(|v| v.parent_name == parent_name)
        .collect();
    children.sort_by(|l, r| l.name.cmp(&r.name));
    for child in children {
        result.push(child.url.clone());

        // recursively get a list of results
        let mut child_results = get_env_order_for(&child.name, environments);
        result.append(&mut child_results);
    }
    result
}

/// Gets a list of environment URLs in order they should be processed
fn get_env_order(environments: &[EnvironmentDetails]) -> Vec<String> {
    let default_url = environments
        .iter()
        .filter(|v| v.name == DEFAULT_ENV_NAME)
        .last()
        .unwrap()
        .url
        .clone();
    let mut result = vec![default_url];
    let mut child_results = get_env_order_for(DEFAULT_ENV_NAME, environments);
    result.append(&mut child_results);
    result
}

fn proc_param_env(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let param_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let mut as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let immediate_parameters = false;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let all_envs = subcmd_args.is_present("all");
    let proj_id = resolved.project_id();

    // assume the provided tag applies to this environment... we cannot use a tag without an
    // environment, so resolve it to an as-of time value
    if let Some(tag_name) = tag {
        let env_id = resolved.environment_id();
        let env_name = resolved.environment_display_name();
        let environments = Environments::new();
        as_of = Some(environments.get_tag_time(rest_cfg, env_id, env_name, &tag_name)?);
    }

    // fetch all environments once, and then determine id's from the same map that is
    // used to resolve the environment names.
    let environments = Environments::new();
    let env_details = environments.get_environment_details(rest_cfg)?;
    let env_url_map = environments.details_to_map(&env_details);
    let url_keys = get_env_order(&env_details);
    let param_values = parameters.get_parameter_environment_map(
        rest_cfg,
        proj_id,
        param_name,
        immediate_parameters,
        !show_secrets,
        as_of,
    )?;

    if param_values.is_empty() {
        error_message(format!("Parameter '{param_name}' was not found"));
        process::exit(10);
    }

    let default_param = ParameterDetails::default();
    let default_env = "Unknown".to_string();
    let mut added = false;
    let mut errors: Vec<String> = vec![];

    let mut table = Table::new("parameter");
    let mut hdr = vec!["Environment", "Value", "FQN", "JMES path"];
    if show_times {
        hdr.push("Created At");
        hdr.push("Modified At");
    }
    table.set_header(&hdr);
    for url in url_keys {
        let env_name = env_url_map.get(&url).unwrap_or(&default_env);
        let details = param_values.get(&url).unwrap_or(&default_param);
        if !details.error.is_empty() {
            errors.push(format_param_error(env_name, &details.error))
        }
        if all_envs
            || details.value != "-"
            || !details.fqn.is_empty()
            || !details.jmes_path.is_empty()
        {
            let mut row = vec![
                env_name.clone(),
                details.value.clone(),
                details.fqn.clone(),
                details.jmes_path.clone(),
            ];
            if show_times {
                row.push(details.created_at.clone());
                row.push(details.modified_at.clone());
            }
            table.add_row(row);
            added = true;
        }
    }
    if !added {
        println!("No values set for '{param_name}' in any environments");
    } else {
        table.render(fmt)?;
    }
    warn_unresolved_params(&errors);

    Ok(())
}

fn proc_param_export(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let starts_with = subcmd_args.value_of("starts_with");
    let ends_with = subcmd_args.value_of("ends_with");
    let contains = subcmd_args.value_of("contains");
    let template_format = subcmd_args.value_of("FORMAT").unwrap();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let export = subcmd_args.is_present("export");
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let options = ParamExportOptions {
        format: ParamExportFormat::from_str(template_format).unwrap(),
        starts_with: starts_with.map(|s| s.to_string()),
        ends_with: ends_with.map(|s| s.to_string()),
        contains: contains.map(|s| s.to_string()),
        export: Some(export),
        secrets: Some(show_secrets),
        as_of,
        tag,
    };
    let body = parameters.export_parameters(rest_cfg, proj_id, env_id, options)?;

    if let Some(body) = body {
        println!("{body}")
    } else {
        println!(
            "Could not export parameters format '{}' from project '{}' in environment '{}'.",
            template_format,
            resolved.project_display_name(),
            resolved.environment_display_name()
        )
    }
    Ok(())
}

fn proc_param_get(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let key = subcmd_args.value_of(KEY_ARG).unwrap();
    let show_details = subcmd_args.is_present("details");
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let evaluate = true; // evaluate the inline content (if applicable)
    let mask_secrets = false; // get the secrets
    let immediate_parameters = subcmd_args.is_present(IMMEDIATE_PARAMETERS_FLAG);
    let parameter = parameters.get_details_by_name(
        rest_cfg,
        proj_id,
        env_id,
        key,
        evaluate,
        immediate_parameters,
        mask_secrets,
        as_of,
        tag,
    )?;

    if let Some(param) = parameter {
        // Treat parameters without values set as if the value were simply empty, since
        // we need to display something sensible to the user.
        if !show_details {
            println!("{}", param.value);
        } else {
            let internal = if param.evaluated {
                param.raw_value
            } else {
                "".to_string()
            };
            printdoc!(
                r#"
                  Name: {}
                  Value: {}
                  Parameter Type: {}
                  Rule Count: {}
                  Source: {}
                  Secret: {}
                  Project URL: {}
                  Description: {}
                  FQN: {}
                  JMES-path: {}
                  Evaluated: {}
                  Raw: {}
                  Parameter-ID: {}
                  Value-ID: {}
                  Environment-ID: {}
                  Created At: {}
                  Modified At: {}
                "#,
                param.key,
                param.value,
                param.param_type,
                param.rules.len(),
                resolved.environment_display_name(),
                param.secret,
                param.project_url,
                param.description,
                param.fqn,
                param.jmes_path,
                param.evaluated,
                internal,
                param.id,
                param.val_id,
                env_id,
                param.created_at,
                param.modified_at,
            );
        }
        if !param.error.is_empty() {
            warning_message(param.error);
        }
    } else {
        println!("The parameter '{key}' could not be found in your organization.");
    }
    Ok(())
}

fn proc_param_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let env_id = resolved.environment_id();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let immediate_parameters = subcmd_args.is_present(IMMEDIATE_PARAMETERS_FLAG);
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let show_rules = subcmd_args.is_present("rules");
    let show_external = subcmd_args.is_present("external");
    let show_evaluated = subcmd_args.is_present("evaluated");
    let show_parents = subcmd_args.is_present("parents");
    let show_children = subcmd_args.is_present("children");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let include_values = (show_values && !show_rules) || show_external || show_evaluated; // don't get values if not needed
    let mut details = parameters.get_parameter_details(
        rest_cfg,
        proj_id,
        env_id,
        immediate_parameters,
        !show_secrets,
        include_values,
        as_of.clone(),
        tag.clone(),
    )?;
    let mut description = "parameters";
    if show_external {
        // when displaying external parameters, only show the external ones
        description = "external parameters";
        details.retain(|x| x.external);
    }
    if show_evaluated {
        description = "evaluated parameters";
        details.retain(|x| x.evaluated);
    }
    if show_rules {
        description = "parameter rules";
        details.retain(|x| !x.rules.is_empty());
    }
    if show_parents {
        description = "parameters from a parent project";
        details.retain(|x| !x.project_url.contains(proj_id));
    }
    if show_children {
        description = "parameters from a child project";

        details.clear(); // starting over with just the children
        let projects = Projects::new();
        let proj_details = projects.get_project_descendants(rest_cfg, proj_name)?;
        // loop through all the projects to find parameters defined in each
        for prj in proj_details {
            let mut child_details = parameters.get_parameter_details(
                rest_cfg,
                &prj.id,
                env_id,
                immediate_parameters,
                !show_secrets,
                include_values,
                as_of.clone(),
                tag.clone(),
            )?;
            child_details.retain(|d| d.project_url.contains(&prj.id));
            details.append(&mut child_details);
        }
    }

    let view_flags = [
        show_rules,
        show_external,
        show_evaluated,
        show_parents,
        show_children,
    ];
    // cannot specify more than 1 of the view flags
    if view_flags.iter().filter(|&x| *x).count() > 1 {
        let msg = concat!(
            "Options for --rules, --external, --evaluated, --parents, and ",
            "--children are mutually exclusive",
        );
        warning_message(msg.to_string());
    } else if details.is_empty() {
        println!("No {description} found in project {proj_name}",);
    } else if !show_values {
        let list = details
            .iter()
            .map(|d| d.key.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else if show_rules {
        // NOTE: do NOT worry about errors, since we're only concerned with params (not values)
        let mut table = Table::new("parameter");
        let mut hdr = vec!["Name", "Param Type", "Rule Type", "Constraint"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        table.set_header(&hdr);
        for entry in details {
            for rule in entry.rules {
                let mut row: Vec<String>;
                row = vec![
                    entry.key.clone(),
                    entry.param_type.to_string(),
                    rule.rule_type.to_string(),
                    rule.constraint,
                ];
                if show_times {
                    row.push(rule.created_at.clone());
                    row.push(rule.modified_at.clone());
                }
                table.add_row(row);
            }
        }
        table.render(fmt)?;
    } else {
        let mut errors: Vec<String> = vec![];
        let mut hdr: Vec<&str>;
        let mut properties: Vec<&str>;

        // setup the table headers and properties
        if show_external {
            hdr = vec!["Name", "FQN", "JMES"];
            properties = vec!["name", "fqn", "jmes-path"];
        } else if show_evaluated {
            hdr = vec!["Name", "Value", "Raw"];
            properties = vec!["name", "value", "raw"];
        } else if show_parents || show_children {
            hdr = vec!["Name", "Value", "Project"];
            properties = vec!["name", "value", "project-name"];
        } else {
            hdr = vec![
                "Name",
                "Value",
                "Source",
                "Param Type",
                "Rules",
                "Type",
                "Secret",
                "Description",
            ];
            properties = vec![
                "name",
                "value",
                "environment",
                "type",
                "rule-count",
                "scope",
                "secret",
                "description",
            ];
        }
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }

        let mut table = Table::new("parameter");
        table.set_header(&hdr);

        for entry in details {
            if !entry.error.is_empty() {
                errors.push(format_param_error(&entry.key, &entry.error));
            }
            table.add_row(entry.get_properties(&properties));
        }
        table.render(fmt)?;

        warn_unresolved_params(&errors);
    }
    Ok(())
}

/// Convenience function to create or update a rule.
fn set_rule_type(
    parameters: &Parameters,
    rest_cfg: &OpenApiConfig,
    details: &ParameterDetails,
    proj_id: &str,
    reuse: bool,
    rule_type: ParamRuleType,
    constraint: &str,
) -> Result<(), ParameterError> {
    let rule_id = details.get_rule_id(rule_type);
    let param_id = &details.id;
    let create = !reuse || rule_id.is_none();
    if create {
        let _ =
            parameters.create_parameter_rule(rest_cfg, proj_id, param_id, rule_type, constraint)?;
    } else {
        // NOTE: not updating the rule_type, so just use None
        let _ = parameters.update_parameter_rule(
            rest_cfg,
            proj_id,
            param_id,
            rule_id.as_ref().unwrap().as_str(),
            None,
            Some(constraint),
        )?;
    }
    Ok(())
}

/// Convenience function to delete a rule of the specified type.
fn delete_rule_type(
    parameters: &Parameters,
    rest_cfg: &OpenApiConfig,
    details: &ParameterDetails,
    proj_id: &str,
    rule_type: ParamRuleType,
) -> Result<(), ParameterError> {
    if let Some(rule_id) = details.get_rule_id(rule_type) {
        let _ = parameters.delete_parameter_rule(rest_cfg, proj_id, &details.id, &rule_id)?;
    }
    Ok(())
}

fn proc_param_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let env_id = resolved.environment_id();
    let prompt_user = subcmd_args.is_present("prompt");
    let generate = subcmd_args.is_present("generate");
    let filename = subcmd_args.value_of("input-file");
    let fqn = subcmd_args.value_of("FQN");
    let jmes_path = subcmd_args.value_of(JMES_PATH_ARG);
    let mut value = subcmd_args.value_of("value");
    let val_str: String;
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let rename = subcmd_args.value_of(RENAME_OPT);
    let final_name = rename.unwrap_or(key_name);
    let mut param_added = false;
    let mut set_action = "Updated";
    let mut env_changed = "".to_string();
    let max_rule = subcmd_args.value_of(RULE_MAX_ARG);
    let min_rule = subcmd_args.value_of(RULE_MIN_ARG);
    let max_len_rule = subcmd_args.value_of(RULE_MAX_LEN_ARG);
    let min_len_rule = subcmd_args.value_of(RULE_MIN_LEN_ARG);
    let regex_rule = subcmd_args.value_of(RULE_REGEX_ARG);
    let delete_max = subcmd_args.is_present(RULE_NO_MAX_ARG);
    let delete_min = subcmd_args.is_present(RULE_NO_MIN_ARG);
    let delete_max_len = subcmd_args.is_present(RULE_NO_MAX_LEN_ARG);
    let delete_min_len = subcmd_args.is_present(RULE_NO_MIN_LEN_ARG);
    let delete_regex = subcmd_args.is_present(RULE_NO_REGEX_ARG);
    let secret: Option<bool> = true_false_option(subcmd_args.value_of("secret"));
    let evaluated: Option<bool> = true_false_option(subcmd_args.value_of("evaluate"));
    let evaluate = false; // no need to evaluate
    let mask_secrets = true; // do not fetch secrets
    let immediate_parameters = false;
    let param_type = subcmd_args.value_of("param-type");
    let create_child = subcmd_args.is_present("create-child");

    // make sure the user did not over-specify
    let mut specified: Vec<bool> = vec![
        jmes_path.is_some() || fqn.is_some(),
        value.is_some(),
        prompt_user,
        generate,
        filename.is_some(),
    ];
    specified.retain(|x| *x);
    if specified.len() > 1 {
        error_message(
            concat!(
                "Conflicting arguments: cannot specify more than one of: prompt, input-file, value, ",
                "generate, or fqn/jmes-path"
            )
            .to_string(),
        );
        process::exit(7);
    }

    // if user asked to be prompted
    if prompt_user {
        println!("Please enter the '{key_name}' value: ");
        val_str = read_password()?;
        value = Some(val_str.as_str());
    } else if let Some(filename) = filename {
        val_str = fs::read_to_string(filename).expect(FILE_READ_ERR);
        value = Some(val_str.as_str());
    } else if generate {
        // TODO: peek ahead at length, type??
        val_str = parameters.generate_password(rest_cfg, 12, None, None, None, None, None, None)?;
        value = Some(val_str.as_str());
    }

    let param_field_update =
        description.is_some() || secret.is_some() || param_type.is_some() || rename.is_some();
    let value_field_update =
        value.is_some() || fqn.is_some() || jmes_path.is_some() || evaluated.is_some();

    // get the original values, so that is not lost
    let mut updated: ParameterDetails;
    if let Some(original) = parameters.get_details_by_name(
        rest_cfg,
        proj_id,
        env_id,
        key_name,
        evaluate,
        immediate_parameters,
        mask_secrets,
        None,
        None,
    )? {
        let parent_project = !original.project_url.contains(proj_id);
        if parent_project && !create_child {
            let projects = Projects::new();
            let source_proj = projects.get_name_from_url(rest_cfg, &original.project_url);
            error_message(format!(
                "Parameter '{key_name}' must be set from project '{source_proj}' -- it is not part of project '{proj_name}'"
            ));
            help_message(format!(
                "If you want to create a parameter in {proj_name}, you can use the --create-child flag. \
                \nThis creates another parameter that inherits values, but generally needs to \nbe \
                managed separately."
            ));
            process::exit(20);
        }

        if parent_project {
            let desc_arg = if description.is_some() {
                description
            } else if !original.description.is_empty() {
                Some(original.description.as_str())
            } else {
                None
            };
            let sec_arg = if secret.is_some() {
                secret
            } else {
                Some(original.secret)
            };
            let type_arg = if param_type.is_some() {
                param_type
            } else {
                Some(original.param_type.as_str())
            };

            param_added = true;
            set_action = "Created child";
            updated = parameters
                .create_parameter(rest_cfg, proj_id, key_name, desc_arg, sec_arg, type_arg)?;
            if updated.param_type == original.param_type {
                for orig_rule in original.rules {
                    let _ = set_rule_type(
                        parameters,
                        rest_cfg,
                        &updated,
                        proj_id,
                        false,
                        orig_rule.rule_type,
                        &orig_rule.constraint,
                    );
                }
            }
            // remove the parameter stuff, so that we create the Value below
            updated.val_id = "".to_string();
            updated.env_name = "".to_string();
            updated.env_url = "".to_string();
        } else if param_field_update {
            // only update if there is something to update
            updated = parameters.update_parameter(
                rest_cfg,
                proj_id,
                &original.id,
                final_name,
                description,
                secret,
                param_type,
            )?;
            // copy a few fields to insure we detect the correct environment
            updated.val_id = original.val_id;
            updated.env_url = original.env_url;
            updated.env_name = original.env_name;
        } else {
            // nothing to update here, but need to copy details
            updated = original;
        }
    } else {
        param_added = true;
        set_action = "Created";
        updated = parameters.create_parameter(
            rest_cfg,
            proj_id,
            key_name,
            description,
            secret,
            param_type,
        )?;
    }

    let param_id = updated.id.as_str();
    let mut rule_errors: Vec<ParameterError> = Vec::new();

    struct RuleDeletion(ParamRuleType, bool);
    let rule_deletions: Vec<RuleDeletion> = vec![
        RuleDeletion(ParamRuleType::Max, delete_max),
        RuleDeletion(ParamRuleType::Min, delete_min),
        RuleDeletion(ParamRuleType::MaxLen, delete_max_len),
        RuleDeletion(ParamRuleType::MinLen, delete_min_len),
        RuleDeletion(ParamRuleType::Regex, delete_regex),
    ];

    for del in rule_deletions {
        if del.1 {
            if let Err(e) = delete_rule_type(parameters, rest_cfg, &updated, proj_id, del.0) {
                rule_errors.push(e);
            }
        }
    }

    // no need to add entries if we've already failed
    if !rule_errors.is_empty() {
        // make sure we don't leave stragglers around
        if param_added {
            // remove the parameter if added
            let _ = parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id);
        }
        for e in rule_errors {
            error_message(e.to_string());
        }
        process::exit(11);
    }

    struct RuleDefinition<'a>(ParamRuleType, Option<&'a str>, bool);
    let rule_defs: Vec<RuleDefinition> = vec![
        RuleDefinition(ParamRuleType::Max, max_rule, !delete_max),
        RuleDefinition(ParamRuleType::Min, min_rule, !delete_min),
        RuleDefinition(ParamRuleType::MaxLen, max_len_rule, !delete_max_len),
        RuleDefinition(ParamRuleType::MinLen, min_len_rule, !delete_min_len),
        RuleDefinition(ParamRuleType::Regex, regex_rule, !delete_regex),
    ];

    for def in rule_defs {
        if let Some(constraint) = def.1 {
            if let Err(e) = set_rule_type(
                parameters, rest_cfg, &updated, proj_id, def.2, def.0, constraint,
            ) {
                rule_errors.push(e);
            }
        }
    }
    if !rule_errors.is_empty() {
        // make sure we don't leave stragglers around
        if param_added {
            // remove the parameter if added
            let _ = parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id);
        }
        for e in rule_errors {
            error_message(e.to_string());
        }
        process::exit(12);
    }

    // don't do anything if there's nothing to do
    if value_field_update {
        env_changed = format!(" for environment '{}'", resolved.environment_display_name());
        // if any existing environment does not match the desired environment
        // or we created a new parameter to override an inherited one
        if !updated.env_url.contains(env_id) || param_added {
            set_action = "Set";
            let value_add_result = parameters.create_parameter_value(
                rest_cfg, proj_id, env_id, param_id, value, fqn, jmes_path, evaluated,
            );
            if let Err(err) = value_add_result {
                if param_added {
                    let _ = parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id);
                }
                return Err(Report::new(err));
            }
        } else {
            parameters.update_parameter_value(
                rest_cfg,
                proj_id,
                param_id,
                &updated.val_id,
                value,
                fqn,
                jmes_path,
                evaluated,
            )?;
        }
    }
    println!(
        "{} parameter '{}' in project '{}'{}.",
        set_action,
        final_name,
        resolved.project_display_name(),
        env_changed,
    );
    Ok(())
}

fn proc_param_unset(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let env_id = resolved.environment_id();
    let env_name = resolved.environment_display_name();
    let result = parameters.delete_parameter_value(rest_cfg, proj_id, env_id, key_name)?;
    match result {
        Some(_) => {
            println!(
                "Removed parameter value '{key_name}' from project '{proj_name}' for environment '{env_name}'.",
            );
        }
        None => {
            warning_message(format!(
                "Did not find parameter value '{key_name}' to delete from project '{proj_name}' for environment '{env_name}'.",
            ));
        }
    };
    Ok(())
}

fn proc_param_push(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG);
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let env_id = resolved.environment_id();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    let steps: Vec<TaskStepDetails>;
    let qualifier: String;
    let include_param_name: bool;
    if let Some(param_name) = key_name {
        if let Some(details) = parameters.get_details_by_name(
            rest_cfg, proj_id, "", param_name, false, false, true, None, None,
        )? {
            steps = parameters.get_task_steps(rest_cfg, proj_id, env_id, &details.id)?;
            qualifier = format!(" for parameter '{param_name}'");
            include_param_name = false;
        } else {
            error_message(format!(
                "Did not find parameter '{param_name}' from project '{proj_name}'.",
            ));
            process::exit(44);
        }
    } else {
        steps = parameters.get_all_task_steps(rest_cfg, proj_id, env_id)?;
        qualifier = "".to_string();
        include_param_name = true;
    }

    if steps.is_empty() {
        println!("No pushes found in project '{proj_name}'{qualifier}.");
    } else if !show_values {
        let list = steps
            .iter()
            .map(|s| s.venue_name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut hdr = vec!["Venue", "Environment", "Result"];
        let mut props = vec!["venue-name", "environment", "result"];
        if include_param_name {
            hdr.insert(1, "Parameter");
            props.insert(1, "parameter");
        }
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            props.push("created-at");
            props.push("modified-at");
        }

        let mut table = Table::new("parameter-push-task-step");
        table.set_header(&hdr);
        for entry in steps {
            table.add_row(entry.get_properties(&props));
        }
        table.render(fmt)?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct DriftDetails {
    pub name: String,
    pub action: String,
    pub current_value: String,
    pub parameter_value: String,
    pub secret: bool,
}

impl DriftDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.name.clone(),
            "action" => self.action.clone(),
            "current" => self.current_value.clone(),
            "server" => self.parameter_value.clone(),
            "secret" => self.secret.to_string(),
            _ => format!("Unhandled property name '{property_name}'"),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

fn maybe_redact(curr_value: &str, is_shown: bool) -> String {
    if is_shown {
        curr_value.to_string()
    } else {
        REDACTED.to_string()
    }
}

fn proc_param_drift(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let immediate_parameters = subcmd_args.is_present(IMMEDIATE_PARAMETERS_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let param_map = parameters.get_parameter_detail_map(
        rest_cfg,
        proj_id,
        env_id,
        immediate_parameters,
        false,
        as_of,
        tag,
    )?;
    let excludes = vec![
        "PATH",
        "HOME",
        "TERM",
        "PWD",
        "OLDPWD",
        "PS1",
        "USER",
        "_",
        // skip all the CLOUDTRUTH_ environment variables for the CLI execution
        "CLOUDTRUTH_API_KEY",
        "CLOUDTRUTH_PROFILE",
        "CLOUDTRUTH_PROJECT",
        "CLOUDTRUTH_ENVIRONMENT",
        "CLOUDTRUTH_SERVER_URL",
        "CLOUDTRUTH_REQUEST_TIMEOUT",
        "CLOUDTRUTH_REST_DEBUG",
        "CLOUDTRUTH_REST_SUCCESS",
        "CLOUDTRUTH_REST_PAGE_SIZE",
    ];
    let env_vars: EnvSettings = env::vars()
        .filter(|(ref k, _)| !excludes.contains(&k.as_str()))
        .collect();

    let mut deltas: Vec<DriftDetails> = vec![];

    // loop through the CloudTruth environment and add items changed in or not present in the shell
    for (k, details) in &param_map {
        let show_this = show_secrets || !details.secret;
        if !env_vars.contains_key(k) {
            deltas.push(DriftDetails {
                name: k.clone(),
                action: "removed".to_string(),
                current_value: "".to_string(),
                parameter_value: maybe_redact(&details.value, show_this),
                secret: details.secret,
            });
            continue;
        }
        let env_value = env_vars.get(k).unwrap();
        if env_value != &details.value {
            deltas.push(DriftDetails {
                name: k.clone(),
                action: "changed".to_string(),
                current_value: maybe_redact(env_value, show_this),
                parameter_value: maybe_redact(&details.value, show_this),
                secret: details.secret,
            })
        }
    }

    // loop through shell environment, and add items not in the CloudTruth environment
    for (k, v) in &env_vars {
        if !param_map.contains_key(k) {
            deltas.push(DriftDetails {
                name: k.clone(),
                action: "added".to_string(),
                current_value: v.clone(),
                parameter_value: "".to_string(),
                secret: false,
            })
        }
    }

    deltas.sort_by_key(|d| d.name.to_lowercase());

    if deltas.is_empty() {
        println!("No drift found.");
    } else if !show_values {
        let list = deltas
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"));
    } else {
        let hdr = vec!["Name", "Difference", "CloudTruth", "Shell"];
        let props = vec!["name", "action", "server", "current"];
        let mut table = Table::new("parameter-drift");
        table.set_header(&hdr);
        for entry in deltas {
            table.add_row(entry.get_properties(&props));
        }
        table.render(fmt)?;
    }
    Ok(())
}

/// Process the 'parameters' sub-command
pub fn process_parameters_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let parameters = Parameters::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_param_list(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_param_get(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_param_set(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_param_delete(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(COPY_SUBCMD) {
        proc_param_copy(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("export") {
        proc_param_export(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("unset") {
        proc_param_unset(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(DIFF_SUBCMD) {
        proc_param_diff(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("environment") {
        proc_param_env(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(PUSH_SUBCMD) {
        proc_param_push(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("drift") {
        proc_param_drift(subcmd_args, rest_cfg, &parameters, resolved)?;
    } else {
        warn_missing_subcommand("parameters");
    }
    Ok(())
}
