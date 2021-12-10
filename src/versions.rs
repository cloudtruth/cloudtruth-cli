use crate::cli::GET_SUBCMD;
use crate::installation::{binary_version, get_latest_version, install_latest_version};
use crate::{error_message, warn_missing_subcommand, warning_message};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;
use version_compare::Version;

fn proc_version_check(subcmd_args: &ArgMatches) -> Result<()> {
    let quiet = subcmd_args.is_present("quiet");
    let latest_str = get_latest_version();
    let latest_ver = Version::from(&latest_str).unwrap();
    let bin_str = binary_version();
    let bin_ver = Version::from(&bin_str).unwrap();

    if bin_ver < latest_ver {
        if !quiet {
            error_message(format!(
                "Version {} is available, running {}",
                latest_ver, bin_ver
            ));
        }
        process::exit(45)
    } else if !quiet {
        let ver = if bin_ver > latest_ver {
            format!("{} (future)", bin_ver)
        } else {
            latest_ver.to_string()
        };
        println!("Running latest {}", ver);
    }

    Ok(())
}

fn proc_version_install(subcmd_args: &ArgMatches) -> Result<()> {
    let force = subcmd_args.is_present("force");
    let quiet = subcmd_args.is_present("quiet");
    let bin_str = binary_version();
    let bin_ver = Version::from(&bin_str).unwrap();
    let mut install = force;

    if !install {
        let latest_str = get_latest_version();
        let latest_ver = Version::from(&latest_str).unwrap();
        install = latest_ver > bin_ver;
    }

    if install {
        install_latest_version(quiet)?;
        println!("Installed the latest CLI.")
    } else {
        warning_message(format!(
            "Already running latest version ({}). You can use --force to re-install.",
            bin_str,
        ))
    }
    Ok(())
}

fn proc_version_get(subcmd_args: &ArgMatches) -> Result<()> {
    let latest = subcmd_args.is_present("latest");
    if latest {
        let ver = get_latest_version();
        println!("Latest CLI version {}", ver);
    } else {
        let ver = binary_version();
        println!("Current CLI version {}", ver)
    }
    Ok(())
}

pub fn process_version_command(subcmd_args: &ArgMatches) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("check") {
        proc_version_check(subcmd_args)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_version_get(subcmd_args)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("install") {
        proc_version_install(subcmd_args)?;
    } else {
        warn_missing_subcommand("versions");
    }
    Ok(())
}
