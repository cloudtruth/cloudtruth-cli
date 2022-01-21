use crate::cli::{CONFIRM_FLAG, FORMAT_OPT};
use crate::database::{Backups, OpenApiConfig};
use crate::lib::{error_message, user_confirm, warn_missing_subcommand, warning_message};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::process;

fn proc_back_snapshot(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    backups: &Backups,
) -> Result<()> {
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    if !confirmed {
        printdoc!(
            r#"
            This action will include unprotected secret values (if any) in output. The resulting
            data needs to be protected appropriately.
            "#
        );
        confirmed = user_confirm("Backup everything to console".to_string(), Some(true));
    }

    if !confirmed {
        warning_message("No backup done".to_string());
    } else {
        let snapshot = backups.data_snapshot(rest_cfg)?;
        match fmt {
            "yaml" => println!("{}", serde_yaml::to_string(&snapshot).unwrap()),
            "json" => println!("{}", serde_json::to_string_pretty(&snapshot).unwrap()),
            _ => {
                error_message(format!("Unsupported format {}", fmt));
                process::exit(55);
            }
        }
    }

    Ok(())
}

/// Process the 'backup' sub-command
pub fn process_backup_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let backups = Backups::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("snapshot") {
        proc_back_snapshot(subcmd_args, rest_cfg, &backups)?;
    } else {
        warn_missing_subcommand("backup");
    }
    Ok(())
}
