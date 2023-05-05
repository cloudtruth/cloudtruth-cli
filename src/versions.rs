use crate::cli::GET_SUBCMD;
use crate::utils::warn_missing_subcommand;
use clap::ArgMatches;
use color_eyre::eyre::Result;
// use std::process;
// use version_compare::Version;

fn proc_version_check(subcmd_args: &ArgMatches) -> Result<()> {
    let _quiet = subcmd_args.is_present("quiet");
    todo!();
    // let latest_str = get_latest_version();
    // let latest_ver = Version::from(&latest_str).unwrap();
    // let bin_str = binary_version();
    // let bin_ver = Version::from(&bin_str).unwrap();
    // if bin_ver < latest_ver {
    //     if !quiet {
    //         error_message(format!(
    //             "Version {latest_ver} is available, running {bin_ver}"
    //         ));
    //     }
    //     process::exit(45)
    // } else if !quiet {
    //     let ver = if bin_ver > latest_ver {
    //         format!("{bin_ver} (future)")
    //     } else {
    //         latest_ver.to_string()
    //     };
    //     println!("Running latest {ver}");
    // }
    // Ok(())
}

fn proc_version_install(subcmd_args: &ArgMatches) -> Result<()> {
    let force = subcmd_args.is_present("force");
    let _quiet = subcmd_args.is_present("quiet");
    // let bin_str = binary_version();
    // let bin_ver = Version::from(&bin_str).unwrap();
    let install = force;

    if !install {
        todo!()
        // let latest_str = get_latest_version();
        // let latest_ver = Version::from(&latest_str).unwrap();
        // install = latest_ver > bin_ver;
    }

    if install {
        todo!()
        // install_latest_version(quiet)?;
        // println!("Installed the latest CLI.")
    } else {
        todo!()
        // warning_message(format!(
        //     "Already running latest version ({bin_str}). You can use --force to re-install.",
        // ))
    }
    // Ok(())
}

fn proc_version_get(subcmd_args: &ArgMatches) -> Result<()> {
    let latest = subcmd_args.is_present("latest");
    if latest {
        todo!();
        // let ver = get_latest_version();
        // println!("Latest CLI version {ver}");
    } else {
        todo!()
        // let ver = binary_version();
        // println!("Current CLI version {ver}")
    }
    // Ok(())
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
