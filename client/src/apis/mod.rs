use reqwest::{Method, Url};
use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl<T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl<T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl<T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl<T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub mod api_api;
pub mod audit_api;
pub mod environments_api;
pub mod import_api;
pub mod integrations_api;
pub mod invitations_api;
pub mod memberships_api;
pub mod organizations_api;
pub mod projects_api;
pub mod serviceaccounts_api;
pub mod users_api;

pub mod configuration;

pub fn handle_serde_error<T>(
    err: serde_json::Error,
    method: &Method,
    url: &Url,
    content: &str,
) -> Error<T> {
    if err.is_data() {
        eprintln!("{} {} error content:\n{}\n", method, url, content);
        if err.line() == 1 {
            let column = err.column();
            let fixed_start = if column < 100 { 0 } else { column - 100 };
            let start = content[..column].rfind('{').unwrap_or(fixed_start);
            // TODO: ignore values containing '}'??
            let end = content[column..].find('}').unwrap_or(column) + column + 1;
            let shortened = &content[start..end];

            let mut fieldname = "Unknown";
            if let Some(end) = content[..column].rfind("\":") {
                if let Some(start) = content[..end].rfind('\"') {
                    fieldname = &content[start + 1..end];
                }
            }

            eprintln!(
                "Context (circa {}):\n  {}\n\nLikely field: {}\n",
                column, shortened, fieldname
            );
        }
    }
    Error::Serde(err)
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

pub(crate) use function;
