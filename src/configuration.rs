use crate::cli::{FORMAT_OPT, LIST_SUBCMD, SECRETS_FLAG, VALUES_FLAG};
use crate::config::Config;
use crate::table::Table;
use crate::{warn_missing_subcommand, REDACTED};
use clap::ArgMatches;
use color_eyre::eyre::Result;

/// Process the 'config' sub-command
pub fn process_config_command(
    subcmd_args: &ArgMatches,
    profile_name: Option<&str>,
    api_key: Option<&str>,
    proj_name: Option<&str>,
    env_name: Option<&str>,
) -> Result<()> {
    if subcmd_args.subcommand_matches("edit").is_some() {
        Config::edit()?;
        let filepath = Config::config_file()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        println!("Edited {}", filepath);
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
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
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("current") {
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
    } else {
        warn_missing_subcommand("configuration")?;
    }
    Ok(())
}
