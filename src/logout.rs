use crate::cli::CONFIRM_FLAG;
use crate::config::Config;
use crate::SEPARATOR;
use crate::{get_api_access_url, user_confirm, warning_message, API_KEY_PAGE};
use clap::ArgMatches;
use color_eyre::eyre::Result;
use indoc::printdoc;

pub fn process_logout_command(subcmd_args: &ArgMatches, config: &Config) -> Result<()> {
    let confirmed = subcmd_args.is_present(CONFIRM_FLAG);
    let api_url = &config.server_url;

    if config.api_key.is_empty() {
        warning_message(format!(
            "No API key is set for profile '{}'",
            config.profile_name
        ))?;
        return Ok(());
    }

    // TODO: instead of using web application, find way to delete API key...
    if let Ok(api_key_url) = get_api_access_url(api_url) {
        let mut open_page = true;
        if !confirmed {
            printdoc!(
                r#"

                {}
                  This removes the API key from profile '{}'.
                  However, it does NOT remove it from the server. You should go to
                  the {} page ({}) and remove it.
                "#,
                SEPARATOR,
                config.profile_name,
                API_KEY_PAGE,
                api_key_url,
            );
            if !user_confirm(format!("Open the {} page", API_KEY_PAGE), Some(true)) {
                open_page = false;
            }
        } else {
            warning_message(format!(
                "Opening {} page ({}) in browser",
                API_KEY_PAGE, api_key_url
            ))?;
        }
        if open_page {
            let open_result = webbrowser::open(&api_key_url);
            if open_result.is_err() {
                printdoc!(
                    r#"
                    "Failed to open browser:
                    {}

                    You can manually open '{}' to delete an API key/token."#,
                    open_result.unwrap_err().to_string(),
                    api_key_url,
                );
            }
        }
    } else {
        warning_message(format!("Unable to determine {} page URL", API_KEY_PAGE))?;
    }

    // NOTE: setting api_key to an empty string forces it to be removed.
    Config::update_profile(&config.profile_name, Some(""), None, None, None)?;
    println!(
        "Updated profile '{}' in {}",
        config.profile_name,
        Config::filename()
    );
    Ok(())
}
