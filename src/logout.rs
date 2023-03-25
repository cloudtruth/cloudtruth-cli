use crate::cli::CONFIRM_FLAG;
use crate::utils::{get_api_access_url, user_confirm, warning_message, API_KEY_PAGE, SEPARATOR};
use clap::ArgMatches;
use cloudtruth_config::Config;
use color_eyre::eyre::Result;
use indoc::printdoc;

pub fn process_logout_command(subcmd_args: &ArgMatches, config: &Config) -> Result<()> {
    let confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    let api_url = &config.server_url;
    let profile_name = &config.profile_name;

    if config.api_key.is_empty() {
        warning_message(format!("No API key is set for profile '{profile_name}'"));
        return Ok(());
    }

    if !confirmed {
        printdoc!(
            r#"

            {}
              This removes the API key from profile '{}'.
              However, it does NOT remove access for that API key.

            "#,
            SEPARATOR,
            profile_name,
        );
        let msg = format!("Do you want to remove the API key from profile '{profile_name}'");
        if !user_confirm(msg, Some(false)) {
            warning_message(format!("Leaving API key in profile '{profile_name}'"));
            return Ok(());
        }
    }

    // TODO: instead of using web application, find way to delete API key...
    if let Ok(api_key_url) = get_api_access_url(api_url) {
        let mut open_page = true;
        if !confirmed {
            printdoc!(
                r#"

                {}
                  Logout does NOT remove access for that API key.
                  You can visit the {} page ({}) to update access.

                "#,
                SEPARATOR,
                API_KEY_PAGE,
                api_key_url,
            );
            if !user_confirm(format!("Open the {API_KEY_PAGE} page"), Some(false)) {
                open_page = false;
            }
        } else {
            warning_message(format!(
                "Opening {API_KEY_PAGE} page ({api_key_url}) in browser"
            ));
        }
        if open_page {
            let open_result = webbrowser::open(&api_key_url);
            if open_result.is_err() {
                printdoc!(
                    r#"
                    "Failed to open browser:
                    {}

                    You can manually open '{}' to delete an API key/token.

                    "#,
                    open_result.unwrap_err().to_string(),
                    api_key_url,
                );
            }
        }
    } else {
        warning_message(format!("Unable to determine {API_KEY_PAGE} page URL"));
    }

    // NOTE: setting api_key to an empty string forces it to be removed.
    Config::update_profile(profile_name, Some(""), None, None, None, None)?;
    println!(
        "Updated profile '{}' in {}",
        profile_name,
        Config::filename()
    );
    Ok(())
}
