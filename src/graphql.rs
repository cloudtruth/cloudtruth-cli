use core::fmt;
use std::sync::Arc;

pub mod prelude {
    use crate::config::Config;
    use crate::graphql::GraphQLError;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    pub fn graphql_request<T: Serialize, R: DeserializeOwned>(
        json: &T,
    ) -> Result<graphql_client::Response<R>, GraphQLError> {
        let client = reqwest::blocking::Client::builder()
            .connection_verbose(true)
            .build()?;
        let config = Config::global();

        let res = client
            .post(&config.server_url)
            .bearer_auth(&config.api_key)
            .json(json)
            .send()?;

        if res.status().is_server_error() {
            Err(GraphQLError::ServerError)
        } else {
            res.json().map_err(GraphQLError::from)
        }
    }
}

#[derive(Debug)]
pub struct UserError {
    pub message: String,
    pub path: Option<Vec<String>>,
}

pub type GraphQLResult<T> = std::result::Result<T, GraphQLError>;

#[derive(Debug, Clone)]
pub enum GraphQLError {
    EnvironmentNotFoundError(String),
    MissingDataError,
    NetworkError(Arc<reqwest::Error>),
    ParameterNotFoundError(String),
    ResponseError(Vec<graphql_client::Error>),
    ServerError,
    ValidationError(String, String),
}

impl GraphQLError {
    pub fn build_logical_error(errors: Vec<UserError>) -> Self {
        if !errors.is_empty() {
            let error = errors.first();
            let mut field = "".to_string();

            if let Some(error) = error {
                if let Some(path) = &error.path {
                    field = path
                        .iter()
                        .filter(|value| **value != "attributes") // Strip out the "attributes" part of the message since it doesn't make sense in a client.
                        .cloned() // Convert &String to String so the values can be joined.
                        .collect::<Vec<String>>()
                        .join(".");
                }
            }

            Self::ValidationError(
                field,
                error.map_or("".to_string(), |error| error.message.clone()),
            )
        } else {
            Self::ServerError
        }
    }
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            GraphQLError::EnvironmentNotFoundError(name) => {
                write!(f, "Unable to find environment '{}'", name)
            }
            GraphQLError::MissingDataError => write!(
                f,
                "GraphQL response did not error, but does not have required data"
            ),
            GraphQLError::NetworkError(_) => write!(f, "Network error performing GraphQL query"),
            GraphQLError::ParameterNotFoundError(key) => {
                write!(f, "Unable to find parameter '{}'", key)
            }
            GraphQLError::ResponseError(errors) => write!(
                f,
                "GraphQL call successfully executed, but the response has errors:\n{}",
                errors
                    .iter()
                    .map(|error| format!("  {}", error))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            GraphQLError::ServerError => write!(f, "There was an error on our server handling your request.\nOur ops team has been alerted and is investigating the issue."),
            GraphQLError::ValidationError(field, message) => write!(f, "There was a problem with a value you supplied: {} {}", field, message)
        }
    }
}

impl From<reqwest::Error> for GraphQLError {
    fn from(error: reqwest::Error) -> Self {
        GraphQLError::NetworkError(Arc::new(error))
    }
}

impl std::error::Error for GraphQLError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GraphQLError::NetworkError(error) => Some(&**error),
            _ => None,
        }
    }
}
