use crate::database::{Api, OpenApiConfig};
use clap::ArgMatches;
use color_eyre::eyre::Result;

pub fn process_schema_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    api: &Api,
) -> Result<()> {
    let fmt = subcmd_args.value_of("format").unwrap();
    let show_version = subcmd_args.is_present("version");
    let show_local = subcmd_args.is_present("local");

    if show_local {
        if show_version {
            let version = api.get_local_schema_version()?;
            println!("{}", version);
        } else {
            let schema = api.get_local_schema(fmt)?;
            println!("{}", schema);
        }
    } else if show_version {
        let version = api.get_schema_version(rest_cfg)?;
        println!("{}", version);
    } else {
        let schema = api.get_schema(rest_cfg, fmt)?;
        println!("{}", schema);
    }
    Ok(())
}
