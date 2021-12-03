use crate::{error_message, warn_missing_subcommand};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;
use version_compare::Version;

pub const LATEST_CHECK_URL: &str =
    "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest";

fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::builder().build().unwrap();
    let request = client
        .request(reqwest::Method::GET, LATEST_CHECK_URL)
        .build()?;
    let mut response = client.execute(request)?;
    let status = response.status();
    let content = response.text()?;

    if !status.is_client_error() && !status.is_server_error() {
        let value: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(dict) = value.as_object() {
            if let Some(tag_value) = dict.get("tag_name") {
                if let Some(tag_str) = tag_value.as_str() {
                    return Ok(tag_str.to_string());
                }
            }
        }
    }
    Ok("0.0.0".to_string())
}

fn proc_version_check(subcmd_args: &ArgMatches) -> Result<()> {
    let quiet = subcmd_args.is_present("quiet");
    let latest_str = get_latest_version()?;
    let latest_ver = Version::from(&latest_str).unwrap();
    let my_str = env!("CARGO_PKG_VERSION");
    let my_ver = Version::from(my_str).unwrap();

    if my_ver < latest_ver {
        if !quiet {
            error_message(format!(
                "Version {} is available, running {}",
                latest_ver, my_str
            ));
        }
        process::exit(45)
    } else if !quiet {
        println!("Running latest {}", latest_str);
    }

    Ok(())
}

pub fn process_version_command(subcmd_args: &ArgMatches) -> Result<()> {
    // TODO: specify timeout?
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("check") {
        proc_version_check(subcmd_args)?;
    } else {
        warn_missing_subcommand("versions");
    }
    Ok(())
}
