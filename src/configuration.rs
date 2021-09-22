use crate::cli::{
    API_KEY_OPT, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, EDIT_SUBCMD, FORMAT_OPT,
    LIST_SUBCMD, NAME_ARG, SECRETS_FLAG, SET_SUBCMD, VALUES_FLAG,
};
use crate::config::Config;
use crate::table::Table;
use crate::{user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM, REDACTED};
use clap::ArgMatches;
use color_eyre::eyre::Result;

fn proc_config_edit() -> Result<()> {
    Config::edit()?;
    println!("Edited {}", Config::filename());
    Ok(())
}

fn proc_config_prof_list(subcmd_args: &ArgMatches) -> Result<()> {
    let details = Config::get_profile_details()?;
    if details.is_empty() {
        println!("No profiles exist in config.");
    } else if !subcmd_args.is_present(VALUES_FLAG) {
        let profile_names: Vec<String> = details.iter().map(|v| v.name.clone()).collect();
        println!("{}", profile_names.join("\n"));
    } else {
        let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
        let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
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
    let values = Config::get_sources(profile_name, api_key, proj_name, env_name)?;

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
            warning_message(format!("Profile '{}' not deleted!", prof_name))?;
        } else {
            Config::delete_profile(prof_name)?;
            println!("Deleted profile '{}'", prof_name);
        }
    } else {
        warning_message(format!("Profile '{}' does not exist!", prof_name))?;
    }
    Ok(())
}

fn proc_config_prof_set(subcmd_args: &ArgMatches) -> Result<()> {
    let prof_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let api_key = subcmd_args.value_of(API_KEY_OPT);
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let project = subcmd_args.value_of("PROJECT");
    let environment = subcmd_args.value_of("ENVIRONMENT");

    if api_key.is_none() && description.is_none() && project.is_none() && environment.is_none() {
        warning_message(format!("Nothing to change for profile '{}'", prof_name))?;
    } else {
        let pre_exists = Config::get_profile_details_by_name(prof_name)?.is_some();
        Config::update_profile(prof_name, api_key, description, environment, project)?;
        let post_exists = Config::get_profile_details_by_name(prof_name)?.is_none();
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
        warn_missing_subcommand("configuration profiles")?;
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
        warn_missing_subcommand("configuration")?;
    }
    Ok(())
}
