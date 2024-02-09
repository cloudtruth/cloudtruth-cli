use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use color_eyre::eyre::Result;
use color_eyre::Report;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::io::{stdin, stdout, Write};
use std::str;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use url::Url;

// The `DEL_CONFIRM` is the default value for delete confirmation across different types
pub const DEL_CONFIRM: Option<bool> = Some(false);
pub const REDACTED: &str = "*****";
pub const FILE_READ_ERR: &str = "Failed to read value from file.";
// old format but server is not accepting it now
// pub const ISO8601: &str = "%Y-%m-%dT%H:%M:%S%.fZ";
pub const ISO8601: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";
pub const SEPARATOR: &str = "=========================";
pub const API_KEY_PAGE: &str = "\"API Access\"";

#[derive(Clone, Debug)]
pub enum ApplicationError {
    InvalidApiUrl(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::InvalidApiUrl(api_url) => {
                write!(f, "No equivalent application URL for API: {api_url}")
            }
        }
    }
}

impl error::Error for ApplicationError {}

/// Print a message to stderr in the specified color.
pub fn stderr_message(message: String, color: Color) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(color));

    stderr.set_color(&color_spec).unwrap_or_default();
    writeln!(&mut stderr, "{message}").unwrap_or_default();
    stderr.reset().unwrap_or_default();
}

/// Print the provided message to stderr in 'Yellow'.
pub fn warning_message<S: Into<String>>(message: S) {
    stderr_message(message.into(), Color::Yellow);
}

/// Print the provided message to stderr in 'Red'.
pub fn error_message<S: Into<String>>(message: S) {
    stderr_message(message.into(), Color::Red);
}

/// Print the provided message to stderr in 'Cyan'.
pub fn help_message<S: Into<String>>(message: S) {
    stderr_message(message.into(), Color::Cyan);
}

pub fn error_no_environment_message(env_name: &str) {
    error_message(format!(
        "The '{env_name}' environment could not be found in your account.",
    ));
}

/// Add "WARN:" prefix to the message, and print it to stderr
pub fn warn_user<S: Into<String>>(message: S) {
    warning_message(format!("WARN: {}", message.into()));
}

/// Simple method for standardizing the message when no sub-command is executed.
pub fn warn_missing_subcommand(command: &str) {
    warn_user(format!("No '{command}' sub-command executed."));
}

/// Method for standardizing message about list of warnings.
pub fn warn_unresolved_params(errors: &[String]) {
    if !errors.is_empty() {
        warning_message(format!(
            "Errors resolving parameters:\n{}\n",
            errors.join("\n")
        ));
    }
}

/// Format the strings in the list of errors
pub fn format_param_error(param_name: &str, param_err: &str) -> String {
    format!("   {param_name}: {param_err}")
}

/// Prompts the user for 'y/n' output.
///
/// If the user answers 'y' (case insensitive), 'true' is returned.
/// If the user answers 'n' (case insensitive), 'false' is returned.
/// The prompt will be repeated upto 3 times if the users does not enter 'y|n'. If the
/// max tries are exceeded, it returns 'false'.
pub fn user_confirm(message: String, default: Option<bool>) -> bool {
    let max_tries = 3;
    let mut confirmed = false;
    let action = match default {
        None => "y/n",
        Some(true) => "Y/n",
        Some(false) => "y/N",
    };

    for _ in 0..max_tries {
        let mut input = String::new();
        print!("{message}? ({action}) ");
        stdout().flush().unwrap();
        let _ = stdin().read_line(&mut input);
        input = input.trim().to_string().to_lowercase();
        if input.is_empty() {
            if let Some(value) = default {
                confirmed = value;
                break;
            }
        }
        if input.as_str() == "y" || input.as_str() == "yes" {
            confirmed = true;
            break;
        }
        if input.as_str() == "n" || input.as_str() == "no" {
            break;
        }
    }
    confirmed
}

/// Get the web application URL for the `API_KEY_PAGE`
pub fn get_api_access_url(api_url: &str) -> Result<String> {
    // remove the any trailing '/'
    let mut api = api_url.to_string();
    if api.ends_with('/') {
        api.truncate(api.len() - 1);
    }
    let api_access_path = "organization/api";
    if api.starts_with("https://localhost:8000") {
        return Ok(format!("https://localhost:7000/{api_access_path}"));
    }
    if api.starts_with("https://api.") && api.ends_with("cloudtruth.io") {
        return Ok(format!(
            "{}/{}",
            api.replace("https://api", "https://app"),
            api_access_path
        ));
    }
    Err(Report::new(ApplicationError::InvalidApiUrl(
        api_url.to_string(),
    )))
}

/// Quick pass at providing a current-time in an acceptable time format for the server.
pub fn current_time() -> String {
    let now = Utc::now();
    now.format(ISO8601).to_string()
}

/// Parse a list of key=value pairs separated by commas (example: foo=bar,bar=qux) into a HashMap
pub fn parse_key_value_pairs(input: &str) -> Option<HashMap<String, String>> {
    input
        .split(',')
        .map(|pair| {
            pair.split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
        })
        .collect()
}

/// Takes an optional CLI argument (`Option<&str>`) attempts to parse it to a valid `DateTime`, and
/// returns the ISO format that the API expects.
///
/// If this is not a recognized date-time format, it will return `None`.
pub fn parse_datetime(input: Option<&str>) -> Option<String> {
    if let Some(orig) = input {
        if let Ok(rfc2822) = DateTime::parse_from_rfc2822(orig) {
            Some(rfc2822.format(ISO8601).to_string())
        } else if let Ok(rfc3339) = DateTime::parse_from_rfc3339(orig) {
            Some(rfc3339.format(ISO8601).to_string())
        } else if let Ok(datetime) = NaiveDateTime::parse_from_str(orig, "%Y-%m-%dT%H:%M:%S%.fZ") {
            Some(datetime.format(ISO8601).to_string())
        } else if let Ok(datetime) = NaiveDateTime::parse_from_str(orig, "%Y-%m-%dT%H:%M:%S%.f") {
            Some(Utc.from_utc_datetime(&datetime).format(ISO8601).to_string())
        } else if let Ok(time_only) = NaiveTime::parse_from_str(orig, "%H:%M:%S%.f") {
            let dt = Utc.from_utc_datetime(&Utc::now().date_naive().and_time(time_only));
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(full_date) = NaiveDate::parse_from_str(orig, "%Y-%m-%d") {
            let dt = Utc.from_utc_datetime(&full_date.and_time(default()));
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(us_date) = NaiveDate::parse_from_str(orig, "%m-%d-%Y") {
            let dt = Utc.from_utc_datetime(&us_date.and_time(default()));
            Some(dt.format(ISO8601).to_string())
        } else if let Ok(us_date) = NaiveDate::parse_from_str(orig, "%m/%d/%Y") {
            let dt = Utc.from_utc_datetime(&us_date.and_time(default()));
            Some(dt.format(ISO8601).to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Returns a tag value, if the input value is not a recognized date-time format.
pub fn parse_tag(input: Option<&str>) -> Option<String> {
    if parse_datetime(input).is_some() {
        None
    } else {
        input.map(String::from)
    }
}

pub fn get_uuid_from_url(url: &str) -> String {
    if let Ok(url) = Url::parse(url) {
        let path_segments: Vec<_> = url.path_segments().unwrap().collect();
        if let Some(uuid_segment) = path_segments.get(3) {
            if uuid_segment.len() == 36 {
                uuid_segment.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }
}

/// Return the default value of a type according to the `Default` trait.
///
/// The type to return is inferred from context; this is equivalent to
/// `Default::default()` but shorter to type.
///
/// See: https://github.com/rust-lang/rust/issues/73014
#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn timedate_and_tag_parsing() {
        // full RFC2822144
        let now = Utc::now();
        let input = now.to_rfc2822();
        let output = parse_datetime(Some(&input)).unwrap();
        assert_eq!(now.format("%FT%T.000000Z").to_string(), output); // no fractional seconds
        assert_eq!(parse_tag(Some(&input)), None);

        // full RFC23339
        let now = Utc::now();
        let input = now.to_rfc3339();
        let output = parse_datetime(Some(&input)).unwrap();
        assert_eq!(now.format(ISO8601).to_string(), output);
        assert_eq!(parse_tag(Some(&input)), None);

        // ISO8601 with Z offset
        let input = Some("2021-07-27T18:34:23.270824Z");
        let expected = Some("2021-07-27T18:34:23.270824Z".into());
        assert_eq!(parse_datetime(input), expected);
        assert_eq!(parse_tag(input), None);

        // ISO8601 - missing trailing Z
        let input = "2021-07-27T18:34:23.270824";
        let output = parse_datetime(Some(input));
        assert!(output.unwrap().contains(input));
        assert_eq!(parse_tag(Some(input)), None);

        // time only, without milliseconds
        let input = Some("02:04:08");
        let output = parse_datetime(input).unwrap();
        assert!(output.contains("02:04:08"));
        assert_eq!(parse_tag(input), None);

        // time only, with milliseconds
        let input = Some("03:05:12.345");
        let output = parse_datetime(input).unwrap();
        assert!(output.contains("T03:05:12.345"));
        assert_eq!(parse_tag(input), None);

        // full date (no time)
        let input = Some("2020-02-02");
        let output = parse_datetime(input).unwrap();
        assert!(output.contains("2020-02-02"));
        assert_eq!(parse_tag(input), None);

        // US date with slashes
        let input = Some("01/19/2021");
        let output = parse_datetime(input).unwrap();
        assert!(output.contains("2021-01-19"));
        assert_eq!(parse_tag(input), None);

        // US date with dashes
        let input = Some("01-19-2021");
        let output = parse_datetime(input).unwrap();
        assert!(output.contains("2021-01-19"));
        assert_eq!(parse_tag(input), None);

        // unfortunately, it lets this through too!
        let input = Some("this is bogus");
        let expected = input.map(String::from);
        assert_eq!(parse_datetime(input), None);
        assert_eq!(parse_tag(input), expected);

        // finally, no option given
        assert_eq!(parse_datetime(None), None);
        assert_eq!(parse_tag(None), None);
    }
}
