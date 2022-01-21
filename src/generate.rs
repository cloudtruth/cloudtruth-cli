use crate::database::{OpenApiConfig, Parameters};
use crate::lib::warn_missing_subcommand;
use clap::ArgMatches;
use color_eyre::eyre::Result;

fn optional_bool(subcmd_args: &ArgMatches, true_flag: &str, false_flag: &str) -> Option<bool> {
    if subcmd_args.is_present(true_flag) {
        return Some(true);
    }
    if subcmd_args.is_present(false_flag) {
        return Some(false);
    }
    None
}

fn proc_gen_password(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let length = subcmd_args
        .value_of("length")
        .unwrap()
        .to_string()
        .parse::<i32>()
        .unwrap();
    let hardware = optional_bool(subcmd_args, "hardware", "no-hardware");
    let uppercase = optional_bool(subcmd_args, "uppercase", "no-uppercase");
    let lowercase = optional_bool(subcmd_args, "lowercase", "no-lowercase");
    let number = optional_bool(subcmd_args, "number", "no-number");
    let symbol = optional_bool(subcmd_args, "symbol", "no-symbol");
    let space = optional_bool(subcmd_args, "space", "no-space");

    let parameters = Parameters::new();
    let password = parameters.generate_password(
        rest_cfg, length, hardware, lowercase, number, space, symbol, uppercase,
    )?;
    println!("{}", password);
    Ok(())
}

/// Process the 'generate' sub-command
pub fn process_generate_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches("password") {
        proc_gen_password(subcmd_args, rest_cfg)?;
    } else {
        warn_missing_subcommand("generate");
    }
    Ok(())
}
