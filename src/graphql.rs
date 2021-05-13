use core::fmt;
use std::sync::Arc;

pub const NO_ORG_ERROR: &str = "Primary organization not found";

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
            .user_agent(concat!("CloudTruth CLI/", env!("CARGO_PKG_VERSION")))
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Operation {
    Create,
    Delete,
    Update,
    Upsert,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Resource {
    Environment,
    Integration,
    Parameter,
    Project,
    Template,
}

pub type GraphQLResult<T> = std::result::Result<T, GraphQLError>;

#[derive(Clone, Debug)]
pub enum GraphQLError {
    AmbiguousIntegrationError(String, String),
    EnvironmentNotFoundError(String),
    IntegrationTypeError(String),
    MissingDataError,
    NetworkError(Arc<reqwest::Error>),
    ParameterNotFoundError(String),
    ProjectNotFoundError(String),
    ResponseError(Vec<graphql_client::Error>),
    ServerError,
    UnauthorizedError(Resource, Operation),
    ValidationError(String, String),
}

impl GraphQLError {
    pub fn build_query_error(
        errors: Vec<graphql_client::Error>,
        resource: Resource,
        operation: Operation,
    ) -> Self {
        let unauthorized = errors
            .iter()
            .find(|error| error.message.contains("not allowed to"));

        if unauthorized.is_some() {
            Self::UnauthorizedError(resource, operation)
        } else {
            Self::ResponseError(errors)
        }
    }

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
            GraphQLError::AmbiguousIntegrationError(name, integration_types) => {
                write!(f, "Found integration named '{}' for types: {}", name, integration_types)
            }
            GraphQLError::EnvironmentNotFoundError(name) => {
                write!(f, "Unable to find environment '{}'.", name)
            }
            GraphQLError::IntegrationTypeError(type_name) => {
                write!(f, "Unable to process integration type '{}'.", type_name)
            }
            GraphQLError::MissingDataError => write!(
                f,
                "GraphQL response did not error, but does not have required data."
            ),
            GraphQLError::NetworkError(_) => write!(f, "Network error performing GraphQL query."),
            GraphQLError::ParameterNotFoundError(key) => {
                write!(f, "Unable to find parameter '{}'.", key)
            },
            GraphQLError::ProjectNotFoundError(name) => {
                write!(f, "Unable to find project '{}'.", name)
            },
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
            GraphQLError::UnauthorizedError(resource, operation) => write!(f, "The access token is not authorized to {} this {}.", operation, resource),
            GraphQLError::ValidationError(field, message) => write!(f, "There was a problem with a value you supplied: {} {}.", field, message),
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

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Operation::Upsert => write!(f, "create or update"),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
