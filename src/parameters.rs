use crate::cli::{
    DELETE_SUBCMD, FORMAT_OPT, GET_SUBCMD, LIST_SUBCMD, RENAME_OPT, SECRETS_FLAG, SET_SUBCMD,
    VALUES_FLAG,
};
use crate::database::{
    Environments, OpenApiConfig, ParamExportFormat, ParamExportOptions, ParameterDetails,
    Parameters,
};
use crate::table::Table;
use crate::{
    error_message, format_param_error, warn_missing_subcommand, warn_unresolved_params, warn_user,
    warning_message, ResolvedIds, FILE_READ_ERR,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use color_eyre::Report;
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
    let key_name = subcmd_args.value_of("KEY").unwrap();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let result = parameters.delete_parameter(rest_cfg, proj_id, env_id, key_name);
    match result {
        Ok(Some(_)) => {
            println!(
                "Successfully removed parameter '{}' from project '{}'.",
                key_name,
                resolved.project_display_name(),
            );
        }
        Ok(None) => {
            println!(
                "Did not find parameter '{}' to delete from project '{}'.",
                key_name,
                resolved.project_display_name(),
            )
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
    let env1_name = subcmd_args.value_of("ENV1").unwrap();
    let env2_name = subcmd_args.value_of("ENV2").unwrap();

    if env1_name == env2_name {
        warning_message("Invalid comparing an environment to itself".to_string())?;
    } else {
        let proj_id = resolved.project_id();

        // fetch all environments once, and then determine id's from the same map that is
        // used to resolve the environment names.
        let environments = Environments::new();
        let env_url_map = environments.get_url_name_map(rest_cfg);
        let env1_id = environments.id_from_map(env1_name, &env_url_map)?;
        let env2_id = environments.id_from_map(env2_name, &env_url_map)?;

        let env1_values = parameters.get_parameter_detail_map(
            rest_cfg,
            &env_url_map,
            proj_id,
            &env1_id,
            !show_secrets,
        )?;
        let env2_values = parameters.get_parameter_detail_map(
            rest_cfg,
            &env_url_map,
            proj_id,
            &env2_id,
            !show_secrets,
        )?;
        let mut param_list: Vec<String> = env1_values.iter().map(|(k, _)| k.clone()).collect();
        param_list.sort_by_key(|l| l.to_lowercase());

        let default_param = ParameterDetails::default();
        let mut added = false;
        let mut table = Table::new("parameter");
        let mut errors: Vec<String> = vec![];
        table.set_header(&["Parameter", env1_name, env2_name]);
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
    }
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
    let key = subcmd_args.value_of("KEY").unwrap();
    let proj_id = resolved.project_id();
    let env_id = resolved.environment_id();
    let parameter = parameters.get_details_by_name(rest_cfg, proj_id, env_id, key, false);

    if let Ok(details) = parameter {
        // Treat parameters without values set as if the value were simply empty, since
        // we need to display something sensible to the user.
        let mut param_value = "".to_string();
        let mut err_msg = "".to_string();
        if let Some(param) = details {
            param_value = param.value;
            err_msg = param.error;
        }
        println!("{}", param_value);
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
    let env_id = resolved.environment_id();
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let mut details = parameters.get_parameter_details(rest_cfg, proj_id, env_id, !show_secrets)?;
    let references = subcmd_args.is_present("dynamic");
    let qualifier = if references { "dynamic " } else { "" };
    if references {
        // when displaying dynamic parameters, only show the dynamic ones
        details.retain(|x| x.dynamic)
    }

    if details.is_empty() {
        println!(
            "No {}parameters found in project {}",
            qualifier,
            resolved.project_display_name()
        );
    } else if !subcmd_args.is_present(VALUES_FLAG) {
        let list = details
            .iter()
            .map(|d| d.key.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
        let mut errors: Vec<String> = vec![];
        let mut table = Table::new("parameter");

        if !references {
            table.set_header(&["Name", "Value", "Source", "Type", "Secret", "Description"]);
        } else {
            table.set_header(&["Name", "FQN", "JMES"]);
        }

        for entry in details {
            if !entry.error.is_empty() {
                errors.push(format_param_error(&entry.key, &entry.error));
            }
            if !references {
                let type_str = if entry.dynamic { "dynamic" } else { "static" };
                let secret_str = if entry.secret { "true" } else { "false" };
                table.add_row(vec![
                    entry.key,
                    entry.value,
                    entry.env_name,
                    type_str.to_string(),
                    secret_str.to_string(),
                    entry.description,
                ]);
            } else {
                table.add_row(vec![entry.key, entry.fqn, entry.jmes_path]);
            }
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
    let key_name = subcmd_args.value_of("KEY").unwrap();
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
    {
        warn_user(
            concat!(
                "Nothing changed. Please provide at least one of: ",
                "description, rename, secret, or value/fqn/jmes-path."
            )
            .to_string(),
        )?;
    } else {
        // get the original values, so that is not lost
        let mut updated: ParameterDetails;
        if let Some(original) =
            parameters.get_details_by_name(rest_cfg, proj_id, env_id, key_name, true)?
        {
            // only update if there is something to update
            if description.is_some() || secret.is_some() || rename.is_some() {
                updated = parameters.update_parameter(
                    rest_cfg,
                    proj_id,
                    &original.id,
                    &final_name,
                    description,
                    secret,
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
            updated =
                parameters.create_parameter(rest_cfg, proj_id, key_name, description, secret)?;
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
    let key_name = subcmd_args.value_of("KEY").unwrap();
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
    } else {
        warn_missing_subcommand("parameters")?;
    }
    Ok(())
}
