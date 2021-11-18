use crate::cli::{
    show_values, CONFIRM_FLAG, DELETE_SUBCMD, DESCRIPTION_OPT, FORMAT_OPT, GET_SUBCMD,
    INVITE_NAME_ARG, LIST_SUBCMD, NAME_ARG, ROLE_ARG, SET_SUBCMD, SHOW_TIMES_FLAG,
};
use crate::database::{Invitations, OpenApiConfig, UserDetails, Users};
use crate::table::Table;
use crate::{error_message, user_confirm, warn_missing_subcommand, warning_message, DEL_CONFIRM};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;
use std::process;

fn print_user(details: &UserDetails) {
    printdoc!(
        r#"
            Name: {}
            Type: {}
            Role: {}
            Email: {}
            Description: {}
            Last Used At: {}
            ID: {}
            User URL: {}
            Created At: {}
            Modified At: {}
        "#,
        details.name,
        details.account_type,
        details.role,
        details.email,
        details.description,
        details.last_used,
        details.id,
        details.user_url,
        details.created_at,
        details.modified_at,
    );
}

fn proc_users_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    users: &Users,
) -> Result<()> {
    let user_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let response = users.get_id(rest_cfg, user_name)?;

    if let Some(user_id) = response {
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete user '{}'", user_name), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("User '{}' not deleted!", user_name));
        } else {
            users.delete_user(rest_cfg, &user_id)?;
            println!("Deleted user '{}'", user_name);
        }
    } else {
        warning_message(format!("User '{}' does not exist!", user_name));
    }
    Ok(())
}

fn proc_users_get(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig, users: &Users) -> Result<()> {
    let user_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let response = users.get_details_by_name(rest_cfg, user_name)?;

    if let Some(details) = response {
        print_user(&details);
    } else {
        error_message(format!("The user '{}' could not be found", user_name));
        process::exit(23);
    }
    Ok(())
}

fn proc_users_current(
    _subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    users: &Users,
) -> Result<()> {
    let details = users.get_current_user(rest_cfg)?;
    print_user(&details);
    Ok(())
}

fn proc_users_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    users: &Users,
) -> Result<()> {
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let show_times = subcmd_args.is_present(SHOW_TIMES_FLAG);
    let show_values = show_values(subcmd_args);
    let details = users.get_user_details(rest_cfg)?;
    if details.is_empty() {
        println!("No users found!");
    } else if !show_values {
        let list = details
            .iter()
            .map(|n| n.name.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let mut hdr = vec!["Name", "Type", "Role", "Email", "Description"];
        let mut properties = vec!["name", "type", "role", "email", "description"];
        if show_times {
            hdr.push("Created At");
            hdr.push("Modified At");
            hdr.push("Last Used At");
            properties.push("created-at");
            properties.push("modified-at");
            properties.push("last-used");
        }

        let mut table = Table::new("user");
        table.set_header(&hdr);
        for entry in details {
            let row = entry.get_properties(&properties);
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_users_set(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig, users: &Users) -> Result<()> {
    let user_name = subcmd_args.value_of(NAME_ARG).unwrap();
    let description = subcmd_args.value_of(DESCRIPTION_OPT);
    let role = subcmd_args.value_of(ROLE_ARG);
    let response = users.get_id(rest_cfg, user_name)?;

    if let Some(user_id) = response {
        if description.is_none() && role.is_none() {
            warning_message(format!(
                "User '{}' not updated: no updated parameters provided",
                user_name
            ));
        } else {
            users.update_user(rest_cfg, &user_id, role, description)?;
            println!("Updated user '{}'", user_name);
        }
    } else {
        let details =
            users.create_user(rest_cfg, user_name, role.unwrap_or("viewer"), description)?;
        println!(
            "Created service account '{}' with api-key:\n{}\n",
            user_name, details.api_key
        );
    }
    Ok(())
}

fn proc_invite_delete(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    invitations: Invitations,
) -> Result<()> {
    let email = subcmd_args.value_of(INVITE_NAME_ARG).unwrap();
    let response = invitations.get_id(rest_cfg, email)?;

    if let Some(invite_id) = response {
        let mut confirmed = subcmd_args.is_present(CONFIRM_FLAG);
        if !confirmed {
            confirmed = user_confirm(format!("Delete invitation for '{}'", email), DEL_CONFIRM);
        }

        if !confirmed {
            warning_message(format!("Invitation for '{}' not deleted!", email));
        } else {
            invitations.delete_invitation(rest_cfg, &invite_id)?;
            println!("Deleted invitation for '{}'", email);
        }
    } else {
        warning_message(format!("Invitation for '{}' does not exist!", email));
    }
    Ok(())
}

fn proc_invite_list(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    invitations: Invitations,
) -> Result<()> {
    let fmt = subcmd_args.value_of(FORMAT_OPT).unwrap();
    let show_values = show_values(subcmd_args);
    let details = invitations.get_invitation_details(rest_cfg)?;
    if details.is_empty() {
        println!("No invitations found!");
    } else if !show_values {
        let list = details
            .iter()
            .map(|n| n.email.clone())
            .collect::<Vec<String>>();
        println!("{}", list.join("\n"))
    } else {
        let hdr = vec!["Email", "Role", "Inviter", "State"];
        let properties = vec!["email", "role", "inviter-name", "state"];
        let mut table = Table::new("invitation");
        table.set_header(&hdr);
        for entry in details {
            let row = entry.get_properties(&properties);
            table.add_row(row);
        }
        table.render(fmt)?;
    }
    Ok(())
}

fn proc_invite_resend(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    invitations: Invitations,
) -> Result<()> {
    let email = subcmd_args.value_of(INVITE_NAME_ARG).unwrap();
    let invite_id = invitations.get_id(rest_cfg, email)?;

    if let Some(invite_id) = invite_id {
        invitations.resend_invitation(rest_cfg, &invite_id)?;
        println!("Resent invitation for '{}'", email);
    } else {
        error_message(format!("Pending invitation for '{}' not found!", email));
        process::exit(29);
    }
    Ok(())
}

fn proc_invite_set(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    invitations: Invitations,
) -> Result<()> {
    let email = subcmd_args.value_of(INVITE_NAME_ARG).unwrap();
    let role = subcmd_args.value_of(ROLE_ARG);
    let response = invitations.get_id(rest_cfg, email)?;

    if let Some(invite_id) = response {
        if role.is_none() {
            warning_message(format!(
                "Invitation for '{}' not updated: no updated parameters provided",
                email
            ));
        } else {
            invitations.update_invitation(rest_cfg, &invite_id, role)?;
            println!("Updated invitation for '{}'", email);
        }
    } else {
        let details = invitations.create_invitation(rest_cfg, email, role.unwrap_or("viewer"))?;
        println!("Sent '{}' invitation as '{}'", details.email, details.role);
    }
    Ok(())
}

fn proc_users_invite_command(
    subcmd_args: &ArgMatches,
    rest_cfg: &OpenApiConfig,
    _users: &Users,
) -> Result<()> {
    let invitations = Invitations::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_invite_delete(subcmd_args, rest_cfg, invitations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_invite_list(subcmd_args, rest_cfg, invitations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("resend") {
        proc_invite_resend(subcmd_args, rest_cfg, invitations)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_invite_set(subcmd_args, rest_cfg, invitations)?;
    } else {
        warn_missing_subcommand("users invitations");
    }
    Ok(())
}

/// Process the 'users' sub-command
pub fn process_users_command(subcmd_args: &ArgMatches, rest_cfg: &OpenApiConfig) -> Result<()> {
    let users = Users::new();
    if let Some(subcmd_args) = subcmd_args.subcommand_matches(DELETE_SUBCMD) {
        proc_users_delete(subcmd_args, rest_cfg, &users)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(GET_SUBCMD) {
        proc_users_get(subcmd_args, rest_cfg, &users)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(LIST_SUBCMD) {
        proc_users_list(subcmd_args, rest_cfg, &users)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches(SET_SUBCMD) {
        proc_users_set(subcmd_args, rest_cfg, &users)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("current") {
        proc_users_current(subcmd_args, rest_cfg, &users)?;
    } else if let Some(subcmd_args) = subcmd_args.subcommand_matches("invitations") {
        proc_users_invite_command(subcmd_args, rest_cfg, &users)?;
    } else {
        warn_missing_subcommand("users");
    }
    Ok(())
}
