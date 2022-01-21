use crate::cli::{
    show_values, API_KEY_OPT, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, EDIT_SUBCMD,
    FORMAT_OPT, LIST_SUBCMD, NAME_ARG, SECRETS_FLAG, SET_SUBCMD,
};
use crate::config::{
    Config, ConfigValue, PARAM_API_KEY, PARAM_ORG, PARAM_PROFILE, PARAM_ROLE, PARAM_USER,
};
use crate::database::{OpenApiConfig, Users};
use crate::lib::{
    error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM, REDACTED,
};
use crate::table::Table;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

fn proc_config_edit() -> Result<()> {
    let filename = Config::filename();
    let orig_content = Config::read_or_create_config()?;
    let mut content = orig_content.clone();

    loop {
        content = edit::edit(content.as_bytes())?;

        let action = if orig_content == content {
            "No changes made to"
        } else {
            "Edited"
        };
        let validation = Config::validate_content(&content);
        if validation.is_ok() {
            Config::update_config(&content)?;
            println!("{} {}", action, filename);
            break;
        }

        warning_message(format!(
            "The provided content is not valid due to:\n{}",
            validation.unwrap_err().to_string()
        ));

        let continue_editing = "Do you want to continue editing".to_string();
        if user_confirm(continue_editing, Some(true)) {
            continue;
        }

        let save_invalid = "Do you want to save the invalid edits".to_string();
        if user_confirm(save_invalid, Some(false)) {
            Config::update_config(&content)?;
            println!("Saving invalid edits to {}", filename);
            break;
        }

        println!("Discarding the invalid edits to {}", filename);
        break;
    }
    Ok(())
}

fn proc_config_prof_list(subcmd_args: &ArgMatches) -> Result<()> {
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let details = Config::get_profile_details()?;

    if details.is_empty() {
        println!("No profiles exist in config.");
    } else if !show_values {
        let profile_names: Vec<String> = details.iter().map(|v| v.name.clone()).collect();
        println!("{}", profile_names.join("\n"));
    } else {
        let mut table = Table::new("profile");
        table.set_header(&["Name", "API", "Environment", "Project", "Description"]);
        for entry in details {
            let mut api_value = "".to_string();
            if let Some(api_key) = entry.api_key {
                if show_secrets {
                    api_value = api_key;
                } else if !api_key.is_empty() {
                    api_value = REDACTED.to_string();
                }
            }
            table.add_row(vec![
                entry.name,
                api_value,
                entry.environment.unwrap_or_default(),
                entry.project.unwrap_or_default(),
                entry.description.unwrap_or_default(),
            ]);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn find_property_value(list: &[ConfigValue], property_name: &str) -> Option<String> {
    for item in list {
        if item.name == property_name {
            if !item.value.is_empty() {
                return Some(item.value.clone());
            }
            break;
        }
    }
    None
}

fn update_property_value(list: &mut [ConfigValue], property_name: &str, value: &str, source: &str) {
    for item in list {
        if item.name == property_name {
            item.value = value.to_string();
            item.source = source.to_string();
            break;
        }
    }
}

fn proc_config_current(
    subcmd_args: &ArgMatches,
    profile_name: Option<&str>,
    api_key: Option<&str>,
    proj_name: Option<&str>,
    env_name: Option<&str>,
) -> Result<()> {
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let show_extended = subcmd_args.is_present("extended");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let mut values = Config::get_sources(profile_name, api_key, proj_name, env_name)?;

    if let Some(api_key) = find_property_value(&values, PARAM_API_KEY) {
        // pull API key and profile name from the list, since the values passed in here are just the CLI arguments,
        // and need to be informed by the environment variables.
        let prof_name = find_property_value(&values, PARAM_PROFILE);
        let config =
            Config::load_config(Some(&api_key), prof_name.as_deref(), env_name, proj_name).unwrap();
        let rest_cfg = OpenApiConfig::from(&config);
        let users = Users::new();

        // NOTE: these only get updated if we can fetch info from the server
        if let Ok(current_user) = users.get_current_user(&rest_cfg) {
            let source = "API key";
            update_property_value(&mut values, PARAM_USER, &current_user.name, source);
            update_property_value(&mut values, PARAM_ROLE, &current_user.role, source);
            update_property_value(&mut values, PARAM_ORG, &current_user.organization, source);
        }
    }

    let mut table = Table::new("profile");
    table.set_header(&["Parameter", "Value", "Source"]);
    for v in values {
        if show_extended || !v.extension {
            let val_str = if show_secrets || !v.secret || v.value.is_empty() {
                v.value
            } else {
                REDACTED.to_string()
            };
            table.add_row(vec![v.name, val_str, v.source]);
        }
    }

    table.render(fmt)?;
    Ok(())
}

fn proc_config_prof_delete(subcmd_args: &ArgMatches) -> Result<()> {
    let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    let prof_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let result = Config::get_profile_details_by_name(prof_name)?;

    if result.is_some() {
        if !confirmed {
            confirmed = user_confirm(format!("Delete profile '{}'", prof_name), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Profile '{}' not deleted!", prof_name));
        } else {
            Config::delete_profile(prof_name)?;
            println!("Deleted profile '{}'", prof_name);
        }
    } else {
        warning_message(format!("Profile '{}' does not exist!", prof_name));
    }
    Ok(())
}

fn proc_config_prof_set(subcmd_args: &ArgMatches) -> Result<()> {
    let prof_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let api_key = subcmd_args.value_of(API_KEY_OPT);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let project = subcmd_args.value_of("PROJECT");
    let environment = subcmd_args.value_of("ENVIRONMENT");
    let source = subcmd_args.value_of("SOURCE");

    // make sure there's a parent profile
    if let Some(source_profile) = source {
        if Config::get_profile_details_by_name(source_profile)?.is_none() {
            error_message(format!(
                "Source profile '{}' does not exist",
                source_profile
            ));
            process::exit(18);
        }
    }

    if api_key.is_none()
        && description.is_none()
        && project.is_none()
        && environment.is_none()
        && source.is_none()
    {
        warning_message(format!("Nothing to change for profile '{}'", prof_name));
    } else {
        let pre_exists = Config::get_profile_details_by_name(prof_name)?.is_some();
        Config::update_profile(
            prof_name,
            api_key,
            description,
            environment,
            project,
            source,
        )?;
        let post_exists = Config::get_profile_details_by_name(prof_name)?.is_some();
        let action = if !post_exists {
            "Deleted"
        } else if !pre_exists {
            "Created"
        } else {
            "Updated"
        };
        println!(
            "{} profile '{}' in '{}'",
            action,
            prof_name,
            Config::filename()
        );
    }
    Ok(())
}

pub fn proc_config_profile_command(subcmd_args: &ArgMatches) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_config_prof_delete(subcmd_args)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_config_prof_list(subcmd_args)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_config_prof_set(subcmd_args)?;
    } else {
        warn_missing_subcommand("configuration profiles");
    }
    Ok(())
}

/// Process the 'config' sub-command
pub fn process_config_command(
    subcmd_args: &ArgMatches,
    profile_name: Option<&str>,
    api_key: Option<&str>,
    proj_name: Option<&str>,
    env_name: Option<&str>,
) -> Result<()> {
    if subcmd_args.subcommand_matches(EDIT_SUBCMD).is_some() {
        proc_config_edit()?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("current") {
        proc_config_current(subcmd_args, profile_name, api_key, proj_name, env_name)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("profiles") {
        proc_config_profile_command(subcmd_args)?;
    } else {
        warn_missing_subcommand("configuration");
    }
    Ok(())
}
