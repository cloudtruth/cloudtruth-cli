use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, LIST_SUBCMD, NAME_ARG,
    PARENT_ARG, RENAME_OPT, RULE_MAX_ARG, RULE_MAX_LEN_ARG, RULE_MIN_ARG, RULE_MIN_LEN_ARG,
    RULE_NO_MAX_ARG, RULE_NO_MAX_LEN_ARG, RULE_NO_MIN_ARG, RULE_NO_MIN_LEN_ARG, RULE_NO_REGEX_ARG,
    RULE_REGEX_ARG, SET_SUBCMD, SHOW_TIMES_FLAG, TREE_SUBCMD,
};
use crate::database::{OpenApiConfig, ParamRuleType, TypeDetails, TypeError, Types};
use crate::table::Table;
use crate::utils::{
    error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM,
};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use std::process;

fn proc_param_type_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let type_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let type_id = types.get_id(rest_cfg, type_name)?;

    if let Some(type_id) = type_id {
        // NOTE: the server is responsible for checking if children exist
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete parameter type '{type_name}'"), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Parameter type '{type_name}' not deleted!"));
        } else {
            types.delete_type(rest_cfg, &type_id)?;
            println!("Deleted parameter type '{type_name}'");
        }
    } else {
        warning_message(format!("Parameter type '{type_name}' does not exist!"));
    }
    Ok(())
}

fn proc_param_type_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let mut details = types.get_type_details(rest_cfg)?;
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_rules = subcmd_args.is_present("rules");
    let show_values = show_values(subcmd_args);
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let mut description = "parameter types";

    if show_rules {
        description = "parameter types with rules";
        details.retain(|x| !x.rules.is_empty());
    }

    if details.is_empty() {
        println!("No {description} found.");
    } else if !show_values {
        let list = details
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"));
    } else if show_rules {
        let mut hdr = vec!["Name", "Parent", "Rule Type", "Constraint"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
        }
        let mut table = Table::new("parameter-type");
        table.set_header(&hdr);
        for entry in details {
            for rule in entry.rules {
                let mut row: Vec<String> = vec![
                    entry.name.clone(),
                    entry.parent_name.clone(),
                    rule.rule_type.to_string(),
                    rule.constraint,
                ];
                if show_times {
                    row.push(rule.created_at.clone());
                    row.push(rule.modified_at.clone());
                }
                table.add_row(row);
            }
        }
        table.render(fmt)?;
    } else {
        let mut hdr = vec!["Name", "Parent", "Rules", "Description"];
        let mut props = vec!["name", "parent-name", "rule-count", "description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            props.push("created-at");
            props.push("modified-at");
        }
        let mut table = Table::new("parameter-type");
        table.set_header(&hdr);
        for entry in details {
            let row = entry.get_properties(&props);
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

/// Convenience function to create or update a rule.
fn set_rule_type(
    types: &Types,
    rest_cfg: &OpenApiConfig,
    details: &TypeDetails,
    reuse: bool,
    rule_type: ParamRuleType,
    constraint: &str,
) -> Result<(), TypeError> {
    let rule_id = details.get_rule_id(rule_type);
    let type_id = &details.id;
    let create = !reuse || rule_id.is_none();
    if create {
        let _ = types.create_type_rule(rest_cfg, type_id, rule_type, constraint)?;
    } else {
        // NOTE: not updating the rule_type, so just use None
        let _ = types.update_type_rule(
            rest_cfg,
            type_id,
            rule_id.as_ref().unwrap().as_str(),
            None,
            Some(constraint),
        )?;
    }
    Ok(())
}

/// Convenience function to delete a rule of the specified type.
fn delete_rule_type(
    types: &Types,
    rest_cfg: &OpenApiConfig,
    details: &TypeDetails,
    rule_type: ParamRuleType,
) -> Result<(), TypeError> {
    if let Some(rule_id) = details.get_rule_id(rule_type) {
        let _ = types.delete_type_rule(rest_cfg, &details.id, &rule_id)?;
    }
    Ok(())
}

fn proc_param_type_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let type_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let rename = subcmd_args.value_of(RENAME_OPT);
    let parent_name = subcmd_args.value_of(PARENT_ARG);
    let mut parent_url: Option<String> = None;
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let details = types.get_details_by_name(rest_cfg, type_name)?;
    let updated: TypeDetails;
    let action: &str;
    let final_name = rename.unwrap_or(type_name);
    let max_rule = subcmd_args.value_of(RULE_MAX_ARG);
    let min_rule = subcmd_args.value_of(RULE_MIN_ARG);
    let max_len_rule = subcmd_args.value_of(RULE_MAX_LEN_ARG);
    let min_len_rule = subcmd_args.value_of(RULE_MIN_LEN_ARG);
    let regex_rule = subcmd_args.value_of(RULE_REGEX_ARG);
    let delete_max = subcmd_args.is_present(RULE_NO_MAX_ARG);
    let delete_min = subcmd_args.is_present(RULE_NO_MIN_ARG);
    let delete_max_len = subcmd_args.is_present(RULE_NO_MAX_LEN_ARG);
    let delete_min_len = subcmd_args.is_present(RULE_NO_MIN_LEN_ARG);
    let delete_regex = subcmd_args.is_present(RULE_NO_REGEX_ARG);
    let type_added: bool;

    if let Some(parent_name) = parent_name {
        if let Some(parent_detail) = types.get_details_by_name(rest_cfg, parent_name)? {
            parent_url = Some(parent_detail.url);
        } else {
            error_message(format!("No parent parameter type '{parent_name}' found"));
            process::exit(46);
        }
    }

    if let Some(details) = details {
        updated = types.update_type(
            rest_cfg,
            final_name,
            &details.id,
            description,
            parent_url.as_deref(),
        )?;
        type_added = false;
        action = "Updated";
    } else {
        if parent_url.is_none() {
            if let Some(parent_detail) = types.get_details_by_name(rest_cfg, "string")? {
                parent_url = Some(parent_detail.url);
            }
        }
        updated = types.create_type(rest_cfg, type_name, description, parent_url.as_deref())?;
        type_added = true;
        action = "Created";
    }

    let mut rule_errors: Vec<TypeError> = Vec::new();
    struct RuleDeletion(ParamRuleType, bool);
    let rule_deletions: Vec<RuleDeletion> = vec![
        RuleDeletion(ParamRuleType::Max, delete_max),
        RuleDeletion(ParamRuleType::Min, delete_min),
        RuleDeletion(ParamRuleType::MaxLen, delete_max_len),
        RuleDeletion(ParamRuleType::MinLen, delete_min_len),
        RuleDeletion(ParamRuleType::Regex, delete_regex),
    ];

    for del in rule_deletions {
        if del.1 {
            if let Err(e) = delete_rule_type(types, rest_cfg, &updated, del.0) {
                rule_errors.push(e);
            }
        }
    }

    // no need to add entries if we've already failed
    if !rule_errors.is_empty() {
        // make sure we don't leave stragglers around
        if type_added {
            // remove the type if added
            let _ = types.delete_type(rest_cfg, &updated.id);
        }
        for e in rule_errors {
            error_message(e.to_string());
        }
        process::exit(47);
    }

    struct RuleDefinition<'a>(ParamRuleType, Option<&'a str>, bool);
    let rule_defs: Vec<RuleDefinition> = vec![
        RuleDefinition(ParamRuleType::Max, max_rule, !delete_max),
        RuleDefinition(ParamRuleType::Min, min_rule, !delete_min),
        RuleDefinition(ParamRuleType::MaxLen, max_len_rule, !delete_max_len),
        RuleDefinition(ParamRuleType::MinLen, min_len_rule, !delete_min_len),
        RuleDefinition(ParamRuleType::Regex, regex_rule, !delete_regex),
    ];

    for def in rule_defs {
        if let Some(constraint) = def.1 {
            if let Err(e) = set_rule_type(types, rest_cfg, &updated, def.2, def.0, constraint) {
                rule_errors.push(e);
            }
        }
    }
    if !rule_errors.is_empty() {
        // make sure we don't leave stragglers around
        if type_added {
            // remove the parameter if added
            let _ = types.delete_type(rest_cfg, &updated.id);
        }
        for e in rule_errors {
            error_message(e.to_string());
        }
        process::exit(48);
    }

    println!("{action} parameter type '{final_name}'");
    Ok(())
}

fn print_children(level: usize, parent_name: &str, list: &[TypeDetails]) {
    let indent = "  ".repeat(level);
    let mut children: Vec<&TypeDetails> = list
        .iter()
        .filter(|x| x.parent_name == parent_name)
        .collect();
    children.sort_by(|l, r| l.name.cmp(&r.name));
    for child in children {
        // print this child
        println!("{}{}", indent, child.name);

        // recursively go through all of it's children
        print_children(level + 1, &child.name, list);
    }
}

fn proc_param_type_tree(
    _subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    types: &Types,
) -> Result<()> {
    let details = types.get_type_details(rest_cfg)?;
    if details.is_empty() {
        println!("No parameter types found.");
    } else {
        for entry in &details {
            if entry.parent_name.is_empty() {
                println!("{}", entry.name);
                print_children(1, &entry.name, &details);
            }
        }
    }
    Ok(())
}

/// Process the 'parameter-types' sub-command
pub fn process_parameter_type_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
) -> Result<()> {
    let types = Types::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_param_type_delete(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_param_type_list(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_param_type_set(subcmd_args, rest_cfg, &types)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(TREE_SUBCMD) {
        proc_param_type_tree(subcmd_args, rest_cfg, &types)?;
    } else {
        warn_missing_subcommand("parameter-types");
    }
    Ok(())
}
