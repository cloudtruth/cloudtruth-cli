use crate::cli::{
    binary_name, AS_OF_ARG, CONFIRM_FLAG, DELETE_SUBCMD, FORMAT_OPT, GET_SUBCMD, KEY_ARG,
    LIST_SUBCMD, RENAME_OPT, SECRETS_FLAG, SET_SUBCMD, SHOW_TIMES_FLAG, VALUES_FLAG,
};
use crate::config::DEFAULT_ENV_NAME;
use crate::database::{
    EnvironmentDetails, Environments, OpenApiConfig, ParamExportFormat, ParamExportOptions,
    ParameterDetails, Parameters,
};
use crate::table::Table;
use crate::{
    error_message, format_param_error, parse_datetime, user_confirm, warn_missing_subcommand,
    warn_unresolved_params, warn_user, warning_message, ResolvedIds, DEL_CONFIRM, FILE_READ_ERR,
};
use clap::ArgMatches;
use cloudtruth_restapi::models::ParameterTypeEnum;
use color_eyre::eyre::Result;
use color_eyre::Report;
use indoc::printdoc;
use rpassword::read_password;
use std::fs;
use std::process;
use std::str::FromStr;

fn proc_param_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    let proj_name = resolved.project_display_name();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let param_id = parameters.get_id(rest_cfg, proj_id, env_id, key_name, None);
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
            format!(
                "Delete parameter '{}' from project '{}'",
                key_name, proj_name
            ),
            DEL_CONFIRM,
        ) {
            return Ok(());
        }
    }

    let result = parameters.delete_parameter_by_id(rest_cfg, proj_id, param_id.unwrap().as_str());
    match result {
        Ok(Some(_)) => {
            println!(
                "Successfully removed parameter '{}' from project '{}'.",
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

fn proc_param_diff(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
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
            "Can specify a maximum of {} environment values.",
            max_len
        ))?;
        return Ok(());
    }
    if as_list.len() > max_len {
        warning_message(format!(
            "Can specify a maximum of {} as-of values.",
            max_len
        ))?;
        return Ok(());
    }

    let env1_name: String;
    let env2_name: String;
    if env_list.len() == 2 {
        env1_name = env_list[0].to_string();
        env2_name = env_list[1].to_string();
    } else if env_list.len() == 1 {
        env1_name = resolved.environment_display_name();
        env2_name = env_list[0].to_string();
    } else {
        env1_name = resolved.environment_display_name();
        env2_name = resolved.environment_display_name();
    }

    let as_of1: Option<String>;
    let as_of2: Option<String>;
    if as_list.len() == 2 {
        as_of1 = parse_datetime(Some(as_list[0]));
        as_of2 = parse_datetime(Some(as_list[1]));
    } else if as_list.len() == 1 {
        // puts the specified time in other column
        as_of1 = None;
        as_of2 = parse_datetime(Some(as_list[0]));
    } else {
        as_of1 = None;
        as_of2 = None;
    }

    if env1_name == env2_name && as_of1 == as_of2 {
        warning_message("Invalid comparing an environment to itself".to_string())?;
        return Ok(());
    }

    let header1: String;
    let header2: String;
    if env1_name == env2_name {
        header1 = as_of1.clone().unwrap_or_else(|| "Current".to_string());
        header2 = as_of2.clone().unwrap_or_else(|| "Unspecified".to_string());
    } else if as_of1 == as_of2 {
        header1 = env1_name.to_string();
        header2 = env2_name.to_string();
    } else {
        header1 = match as_of1 {
            Some(ref a) => format!("{} ({})", env1_name, a),
            _ => env1_name.to_string(),
        };
        header2 = match as_of2 {
            Some(ref a) => format!("{} ({})", env2_name, a),
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
        &env_url_map,
        proj_id,
        &env1_id,
        !show_secrets,
        as_of1,
    )?;
    let env2_values = parameters.get_parameter_detail_map(
        rest_cfg,
        &env_url_map,
        proj_id,
        &env2_id,
        !show_secrets,
        as_of2,
    )?;

    // get the names from both lists to make sure we get the added/deleted parameters, too
    let mut param_list: Vec<String> = env1_values.iter().map(|(k, _)| k.clone()).collect();
    param_list.append(&mut env2_values.iter().map(|(k, _)| k.clone()).collect());
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
    warn_unresolved_params(&errors)?;
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
    resolved: &ResolvedIds,
) -> Result<()> {
    let param_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let all_envs = subcmd_args.is_present("all");
    let proj_id = resolved.project_id();

    // fetch all environments once, and then determine id's from the same map that is
    // used to resolve the environment names.
    let environments = Environments::new();
    let env_details = environments.get_environment_details(rest_cfg)?;
    let env_url_map = environments.details_to_map(&env_details);
    let url_keys = get_env_order(&env_details);
    let param_values = parameters.get_parameter_environment_map(
        rest_cfg,
        &env_url_map,
        proj_id,
        param_name,
        !show_secrets,
        as_of,
    )?;

    if param_values.is_empty() {
        error_message(format!("Parameter '{}' was not found", param_name))?;
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
        println!("No values set for '{}' in any environments", param_name);
    } else {
        table.render(fmt)?;
    }
    warn_unresolved_params(&errors)?;

    Ok(())
}

fn proc_param_export(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let starts_with = subcmd_args.value_of("starts_with");
    let ends_with = subcmd_args.value_of("ends_with");
    let contains = subcmd_args.value_of("contains");
    let template_format = subcmd_args.value_of("FORMAT").unwrap();
    let export = subcmd_args.is_present("export");
    let secrets = subcmd_args.is_present(SECRETS_FLAG);
    let options = ParamExportOptions {
        format: ParamExportFormat::from_str(template_format).unwrap(),
        starts_with: starts_with.map(|s| s.to_string()),
        ends_with: ends_with.map(|s| s.to_string()),
        contains: contains.map(|s| s.to_string()),
        export: Some(export),
        secrets: Some(secrets),
    };
    let body = parameters.export_parameters(rest_cfg, proj_id, env_id, options)?;

    if let Some(body) = body {
        println!("{}", body)
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
    resolved: &ResolvedIds,
) -> Result<()> {
    let key = subcmd_args.value_of(KEY_ARG).unwrap();
    let show_details = subcmd_args.is_present("details");
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let parameter = parameters.get_details_by_name(rest_cfg, proj_id, env_id, key, false, as_of);

    if let Ok(details) = parameter {
        // Treat parameters without values set as if the value were simply empty, since
        // we need to display something sensible to the user.
        let mut param_value = "".to_string();
        let mut err_msg = "".to_string();
        if let Some(ref param) = details {
            param_value = param.value.clone();
            err_msg = param.error.clone();
        }
        if !show_details {
            println!("{}", param_value);
        } else if let Some(param) = details {
            printdoc!(
                r#"
                  Name: {}:
                  Value: {}
                  Parameter Type: {}
                  Rule Count: {}
                  Source: {}
                  Secret: {}
                  Description: {}
                  FQN: {}
                  JMES-path: {}
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
                param.description,
                param.fqn,
                param.jmes_path,
                param.id,
                param.val_id,
                env_id,
                param.created_at,
                param.modified_at,
            );
        }
        if !err_msg.is_empty() {
            warning_message(err_msg)?;
        }
    } else {
        println!(
            "The parameter '{}' could not be found in your organization.",
            key
        );
    }
    Ok(())
}

fn proc_param_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let proj_id = resolved.project_id();
    let proj_name = resolved.project_display_name();
    let env_id = resolved.environment_id();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_rules = subcmd_args.is_present("rules");
    let show_values =
        subcmd_args.is_present(VALUES_FLAG) || show_secrets || show_times || show_rules;
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let mut details =
        parameters.get_parameter_details(rest_cfg, proj_id, env_id, !show_secrets, as_of)?;
    let references = subcmd_args.is_present("dynamic");
    let qualifier = if references { "dynamic " } else { "" };
    if references {
        // when displaying dynamic parameters, only show the dynamic ones
        details.retain(|x| x.dynamic)
    }

    if show_rules && references {
        warning_message("Options for --dynamic and --rules are mutually exclusive".to_string())?;
    } else if details.is_empty() {
        println!("No {}parameters found in project {}", qualifier, proj_name,);
    } else if !show_values {
        let list = details
            .iter()
            .map(|d| d.key.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else if show_rules {
        // NOTE: do NOT worry about errors, since we're only concerned with params (not values)
        let mut table = Table::new("parameter");
        let mut hdr = vec!["Name", "Rule Type", "Constraint"];
        let mut added = false;
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        table.set_header(&hdr);
        for entry in details {
            if entry.rules.is_empty() {
                continue;
            }

            for rule in entry.rules {
                let mut row: Vec<String>;
                row = vec![
                    entry.key.clone(),
                    rule.rule_type.to_string(),
                    rule.constraint,
                ];
                if show_times {
                    row.push(rule.created_at.clone());
                    row.push(rule.modified_at.clone());
                }
                table.add_row(row);
                added = true;
            }
        }
        if added {
            table.render(fmt)?;
        } else {
            println!("No parameter rules found in project '{}'", proj_name)
        }
    } else {
        let mut errors: Vec<String> = vec![];
        let mut table = Table::new("parameter");
        let mut hdr = if !references {
            vec![
                "Name",
                "Value",
                "Source",
                "Param Type",
                "Rules",
                "Type",
                "Secret",
                "Description",
            ]
        } else {
            vec!["Name", "FQN", "JMES"]
        };
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        table.set_header(&hdr);

        for entry in details {
            if !entry.error.is_empty() {
                errors.push(format_param_error(&entry.key, &entry.error));
            }
            let mut row: Vec<String>;
            if !references {
                let type_str = if entry.dynamic { "dynamic" } else { "static" };
                let secret_str = if entry.secret { "true" } else { "false" };
                row = vec![
                    entry.key,
                    entry.value,
                    entry.env_name,
                    entry.param_type,
                    entry.rules.len().to_string(),
                    type_str.to_string(),
                    secret_str.to_string(),
                    entry.description,
                ];
            } else {
                row = vec![entry.key, entry.fqn, entry.jmes_path];
            }
            if show_times {
                row.push(entry.created_at);
                row.push(entry.modified_at);
            }
            table.add_row(row);
        }
        table.render(fmt)?;

        warn_unresolved_params(&errors)?;
    }
    Ok(())
}

fn proc_param_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let prompt_user = subcmd_args.is_present("prompt");
    let filename = subcmd_args.value_of("input-file");
    let fqn = subcmd_args.value_of("FQN");
    let jmes_path = subcmd_args.value_of("JMES");
    let mut value = subcmd_args.value_of("value");
    let val_str: String;
    let description = subcmd_args.value_of("description");
    let rename = subcmd_args.value_of(RENAME_OPT);
    let final_name = rename.unwrap_or(key_name);
    let mut param_added = false;
    let secret: Option<bool> = match subcmd_args.value_of("secret") {
        Some("false") => Some(false),
        Some("true") => Some(true),
        _ => None,
    };
    let param_type = match subcmd_args.value_of("param-type") {
        Some("string") => Some(ParameterTypeEnum::String),
        Some("integer") => Some(ParameterTypeEnum::Integer),
        Some("bool") => Some(ParameterTypeEnum::Bool),
        None => None,
        Some(x) => {
            warning_message(format!("Unhandled type '{}'", x))?;
            None
        }
    };

    // make sure the user did not over-specify
    if (jmes_path.is_some() || fqn.is_some())
        && (value.is_some() || prompt_user || filename.is_some())
    {
        error_message(
            concat!(
                "Conflicting arguments: cannot specify prompt/input-file/value, ",
                "and fqn/jmes-path"
            )
            .to_string(),
        )?;
        process::exit(7);
    }

    // if user asked to be prompted
    if prompt_user {
        println!("Please enter the '{}' value: ", key_name);
        val_str = read_password()?;
        value = Some(val_str.as_str());
    } else if let Some(filename) = filename {
        val_str = fs::read_to_string(filename).expect(FILE_READ_ERR);
        value = Some(val_str.as_str());
    }

    // make sure there is at least one parameter to updated
    if description.is_none()
        && secret.is_none()
        && value.is_none()
        && jmes_path.is_none()
        && fqn.is_none()
        && rename.is_none()
        && param_type.is_none()
    {
        warn_user(
            concat!(
                "Nothing changed. Please provide at least one of: ",
                "description, rename, secret, type, or value/fqn/jmes-path."
            )
            .to_string(),
        )?;
    } else {
        // get the original values, so that is not lost
        let mut updated: ParameterDetails;
        if let Some(original) =
            parameters.get_details_by_name(rest_cfg, proj_id, env_id, key_name, true, None)?
        {
            // only update if there is something to update
            if description.is_some() || secret.is_some() || rename.is_some() || param_type.is_some()
            {
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
            updated = parameters.create_parameter(
                rest_cfg,
                proj_id,
                key_name,
                description,
                secret,
                param_type,
            )?;
        }

        // don't do anything if there's nothing to do
        if value.is_some() || fqn.is_some() || jmes_path.is_some() {
            let param_id = updated.id.as_str();
            // if any existing environment does not match the desired environment
            if !updated.env_url.contains(env_id) {
                let value_add_result = parameters.create_parameter_value(
                    rest_cfg, proj_id, env_id, param_id, value, fqn, jmes_path,
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
                )?;
            }
        }
        println!(
            "Successfully updated parameter '{}' in project '{}' for environment '{}'.",
            final_name,
            resolved.project_display_name(),
            resolved.environment_display_name(),
        );
    }
    Ok(())
}

fn proc_param_unset(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    let key_name = subcmd_args.value_of(KEY_ARG).unwrap();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let result = parameters.delete_parameter_value(rest_cfg, proj_id, env_id, key_name);
    match result {
        Ok(Some(_)) => {
            println!(
                "Successfully removed parameter value '{}' from project '{}' for environment '{}'.",
                key_name,
                resolved.project_display_name(),
                resolved.environment_display_name()
            );
        }
        Ok(None) => {
            println!(
                "Did not find parameter value '{}' to delete from project '{}' for environment '{}'.",
                key_name,
                resolved.project_display_name(),
                resolved.environment_display_name()
            )
        }
        _ => {
            println!(
                "Failed to remove parameter value '{}' from project '{}' for environment '{}'.",
                key_name,
                resolved.project_display_name(),
                resolved.environment_display_name()
            );
        }
    };
    Ok(())
}

/// Process the 'parameters' sub-command
pub fn process_parameters_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    parameters: &Parameters,
    resolved: &ResolvedIds,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_param_list(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_param_get(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_param_set(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_param_delete(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("export") {
        proc_param_export(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("unset") {
        proc_param_unset(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("differences") {
        proc_param_diff(subcmd_args, rest_cfg, parameters, resolved)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("environment") {
        proc_param_env(subcmd_args, rest_cfg, parameters, resolved)?;
    } else {
        warn_missing_subcommand("parameters")?;
    }
    Ok(())
}
