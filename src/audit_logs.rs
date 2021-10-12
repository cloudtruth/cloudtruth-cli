use crate::cli::{FORMAT_OPT, LIST_SUBCMD};
use crate::database::{to_object_type, AuditLogs, OpenApiConfig};
use crate::table::Table;
use crate::warn_missing_subcommand;
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;

fn proc_audit_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    audit_logs: &AuditLogs,
) -> Result<()> {
    let object_type = to_object_type(subcmd_args.value_of("type")).map(|x| x.to_server_string());
    let action = subcmd_args.value_of("action");
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let details = audit_logs.get_audit_log_details(rest_cfg, object_type.as_deref(), action)?;

    if details.is_empty() {
        println!("No audit log entries found");
    } else {
        let hdr = vec!["Time", "Object Name", "Type", "Action", "User"];
        let mut table = Table::new("audit-logs");
        table.set_header(&hdr);
        for entry in details {
            let row = vec![
                entry.timestamp,
                entry.object_name,
                entry.object_type.to_string(),
                entry.action,
                entry.user,
            ];
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_audit_summary(
    _subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    audit_logs: &AuditLogs,
) -> Result<()> {
    let summary = audit_logs.get_audit_log_summary(rest_cfg)?;
    printdoc!(
        r#"
          Record count: {}
          Earliers record: {}
          Policy:
            Maximum records: {}
            Maximum days: {}
        "#,
        summary.total_records,
        summary.earliest,
        summary.max_records,
        summary.max_days,
    );
    Ok(())
}

pub fn process_audit_log_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    audit_logs: &AuditLogs,
) -> Result<()> {
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_audit_list(subcmd_args, rest_cfg, audit_logs)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("summary") {
        proc_audit_summary(subcmd_args, rest_cfg, audit_logs)?;
    } else {
        warn_missing_subcommand("audit-logs")?;
    }
    Ok(())
}
