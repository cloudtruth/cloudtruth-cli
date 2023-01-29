use crate::cli::{DIFF_SUBCMD, FORMAT_OPT};
use crate::database::{Api, OpenApiConfig};
use crate::utils::warn_missing_subcommand;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use similar::TextDiff;
use std::io;

fn proc_schema_diff(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig, api: &Api) -> Result<()> {
    let show_version = subcmd_args.is_present("version");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    let header1 = "CLI";
    let header2 = "Server";
    let server: String;
    let local: String;
    if show_version {
        server = api.get_schema_version(rest_cfg)?;
        local = api.get_local_schema_version()?;
    } else {
        server = api.get_schema(rest_cfg, fmt)?;
        local = api.get_local_schema(fmt)?;
    }
    let diff = TextDiff::from_lines(&local, &server);
    diff.unified_diff()
        .header(header1, header2)
        .to_writer(io::stdout())?;
    Ok(())
}

fn proc_schema_local(subcmd_args: &ArgMatches, _rest_cfg: &OpenApiConfig, api: &Api) -> Result<()> {
    let show_version = subcmd_args.is_present("version");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    if show_version {
        let version = api.get_local_schema_version()?;
        println!("{version}");
    } else {
        let schema = api.get_local_schema(fmt)?;
        println!("{schema}");
    }
    Ok(())
}

fn proc_schema_server(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig, api: &Api) -> Result<()> {
    let show_version = subcmd_args.is_present("version");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();

    if show_version {
        let version = api.get_schema_version(rest_cfg)?;
        println!("{version}");
    } else {
        let schema = api.get_schema(rest_cfg, fmt)?;
        println!("{schema}");
    }
    Ok(())
}

pub fn process_schema_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let api = Api::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DIFF_SUBCMD) {
        proc_schema_diff(subcmd_args, rest_cfg, &api)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("local") {
        proc_schema_local(subcmd_args, rest_cfg, &api)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("server") {
        proc_schema_server(subcmd_args, rest_cfg, &api)?;
    } else {
        warn_missing_subcommand("schema");
    }
    Ok(())
}
