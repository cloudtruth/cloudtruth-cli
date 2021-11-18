use crate::cli::{FORMAT_OPT, LIST_SUBCMD};
use crate::database::{AuditLogs, OpenApiConfig, Users};
use crate::table::Table;
use crate::{error_message, help_message, parse_datetime, warn_missing_subcommand};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::process;

const OBJECT_TYPE_VALUES: &[&str] = &[
    "Aws",
    "Environment",
    "Github",
    "Invitation",
    "Membership",
    "Organization",
    "Parameter",
    "Rule",
    "Project",
    "Pull",
    "Push",
    "ServiceAccount",
    "Tag",
    "Task",
    "Template",
    "Value",
];

/// Print a consistent `error_message()`
fn invalid_time_format(arg: &str) {
    error_message(format!("Invalid '{}' value", arg));
}

fn resolve_object_type(input: &str) -> String {
    let lowerin = input.to_lowercase();
    for v in OBJECT_TYPE_VALUES {
        if v.to_lowercase() == lowerin {
            return v.to_string();
        }
    }
    if lowerin == "service-account" {
        return "ServiceAccount".to_string();
    }
    input.to_string()
}

fn proc_audit_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    audit_logs: &AuditLogs,
) -> Result<()> {
    let action = subcmd_args.value_of("action");
    let object_type = subcmd_args.value_of("object-type").map(resolve_object_type);
    let before = parse_datetime(subcmd_args.value_of("before"));
    let after = parse_datetime(subcmd_args.value_of("after"));
    let name = subcmd_args.value_of("contains");
    let username = subcmd_args.value_of("username");
    let max_entries = subcmd_args
        .value_of("max-entries")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let bad_before = subcmd_args.occurrences_of("before") > 0 && before.is_none();
    let bad_after = subcmd_args.occurrences_of("after") > 0 && after.is_none();

    if bad_before || bad_after {
        if bad_before {
            invalid_time_format("--before");
        }
        if bad_after {
            invalid_time_format("--after");
        }
        process::exit(34);
    }

    let mut user_id = None;
    if let Some(uname) = username {
        let users = Users::new();
        user_id = users.get_id(rest_cfg, uname)?;
        if user_id.is_none() {
            error_message(format!("User '{}' not found.", uname));
            process::exit(35);
        }
    }

    let details = audit_logs.get_audit_log_details(
        rest_cfg,
        object_type.as_deref(),
        action,
        name,
        max_entries,
        before,
        after,
        user_id.as_deref(),
    )?;

    if details.is_empty() {
        let mut constraints: Vec<String> = vec![];
        if let Some(o) = object_type {
            constraints.push(format!("type=={}", o));
            if !OBJECT_TYPE_VALUES.contains(&o.as_str()) {
                help_message(format!(
                    "The specified --type is not one of the recognized values: {}",
                    OBJECT_TYPE_VALUES.join(", ")
                ));
            }
        }
        if let Some(n) = name {
            constraints.push(format!("name-contains '{}'", n));
        }
        if let Some(a) = action {
            constraints.push(format!("action=={}", a));
        }
        if constraints.is_empty() {
            println!("No audit log entries found");
        } else {
            println!(
                "No audit log entries found matching {}",
                constraints.join(", ")
            );
        }
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
          Earliest record: {}
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

pub fn process_audit_log_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let audit_logs = AuditLogs::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_audit_list(subcmd_args, rest_cfg, &audit_logs)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("summary") {
        proc_audit_summary(subcmd_args, rest_cfg, &audit_logs)?;
    } else {
        warn_missing_subcommand("audit-logs");
    }
    Ok(())
}
