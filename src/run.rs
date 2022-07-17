use crate::cli::AS_OF_ARG;
use crate::database::{OpenApiConfig, Parameters, ResolvedDetails};
use crate::lib::{
    format_param_error, parse_datetime, parse_tag, warn_missing_subcommand, warn_unresolved_params,
    warn_user,
};
use crate::subprocess::{EnvSettings, Inheritance, SubProcess};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;
use std::str::FromStr;

/// Process the 'run' sub-command
pub fn process_run_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    resolved: &ResolvedDetails,
) -> Result<()> {
    let mut sub_proc = SubProcess::new();
    let as_of = parse_datetime(subcmd_args.value_of(AS_OF_ARG));
    let tag = parse_tag(subcmd_args.value_of(AS_OF_ARG));
    let mut arguments: Vec<String>;
    let command: String;

    let parameters = Parameters::new();
    let param_map = parameters.get_parameter_values(
        rest_cfg,
        resolved.project_id(),
        resolved.environment_id(),
        false,
        true,
        as_of,
        tag,
    )?;
    let mut ct_vars = EnvSettings::new();
    let mut errors: Vec<String> = vec![];
    for (k, v) in param_map {
        ct_vars.insert(k.clone(), v.value.clone());
        if !v.error.is_empty() {
            errors.push(format_param_error(&k, &v.error))
        }
    }
    sub_proc.set_cloudtruth_environment(ct_vars);

    if subcmd_args.is_present("command") {
        command = subcmd_args.value_of("command").unwrap().to_string();
        arguments = vec![];
    } else if subcmd_args.is_present("arguments") {
        arguments = subcmd_args.values_of_lossy("arguments").unwrap();
        command = arguments.remove(0);
        if command.contains(' ') {
            warn_user("command contains spaces, and may fail.".to_string());
            let mut reformed = format!("{} {}", command, arguments.join(" "));
            reformed = reformed.replace('$', "\\$");
            println!(
                "Try using 'cloudtruth run --command \"{}\"'",
                reformed.trim()
            );
        }
    } else {
        warn_missing_subcommand("run");
        process::exit(0);
    }

    // NOTE: do this before running the sub-process, since it could be a long-running task
    warn_unresolved_params(&errors);

    // Setup the environment for the sub-process.
    let inherit = Inheritance::from_str(subcmd_args.value_of("inheritance").unwrap()).unwrap();
    let overrides = subcmd_args.values_of_lossy("set").unwrap_or_default();
    let removals = subcmd_args.values_of_lossy("remove").unwrap_or_default();
    let permissive = subcmd_args.is_present("permissive");
    let strict = subcmd_args.is_present("strict");
    sub_proc.set_environment(resolved, inherit, &overrides, &removals, strict)?;
    if !permissive {
        sub_proc.remove_ct_app_vars();
    }
    sub_proc.run_command(command.as_str(), &arguments)?;

    Ok(())
}
