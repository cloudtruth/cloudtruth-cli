use crate::cli::{FORMAT_OPT, SECRETS_FLAG, SHOW_TIMES_FLAG};
use crate::database::{Imports, OpenApiConfig};
use crate::lib::{warn_missing_subcommand, FILE_READ_ERR};
use crate::table::Table;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::fs;

fn proc_import_parameters(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    imports: &Imports,
) -> Result<()> {
    let proj_name = subcmd_args.value_of("project").unwrap();
    let env_name = subcmd_args.value_of("environment");
    let filename = subcmd_args.value_of("file").unwrap();
    let preview = subcmd_args.is_present("preview");
    let ignores: Vec<&str> = subcmd_args
        .values_of("ignore-param")
        .unwrap_or_default()
        .collect();
    let secret_params: Vec<&str> = subcmd_args
        .values_of("secret-param")
        .unwrap_or_default()
        .collect();
    let inherit = !subcmd_args.is_present("no-inherit");
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_secrets = subcmd_args.is_present(SECRETS_FLAG);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let text = fs::read_to_string(filename).expect(FILE_READ_ERR);

    let details = imports.import_parameters(
        rest_cfg,
        proj_name,
        env_name,
        &text,
        &secret_params,
        &ignores,
        inherit,
        preview,
        !show_secrets,
    )?;
    if details.is_empty() {
        println!("No parameters to import.");
    } else {
        let mut hdr = vec!["Name", "Value", "Change", "Project", "Environment"];
        let mut properties = vec!["name", "value", "action", "project", "environment"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            properties.push("created-at");
            properties.push("modified-at");
        }

        let mut table = Table::new("parameter");
        table.set_header(&hdr);
        for entry in details {
            table.add_row(entry.get_properties(&properties));
        }
        table.render(fmt)?;
    }
    Ok(())
}

/// Process the 'importer' sub-command
pub fn process_import_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let imports = Imports::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("parameters") {
        proc_import_parameters(subcmd_args, rest_cfg, &imports)?;
    } else {
        warn_missing_subcommand("import");
    }
    Ok(())
}
