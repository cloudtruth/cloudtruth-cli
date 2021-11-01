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

    let schema = if show_version {
        // for now, just provide server version
        // TODO: also provide client version
        api.get_schema_version(rest_cfg)?
    } else {
        api.get_schema(rest_cfg, fmt)?
    };
    println!("{}", schema);
    Ok(())
}
