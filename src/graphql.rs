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
        let client = reqwest::blocking::Client::new();
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

pub type GraphQLResult<T> = std::result::Result<T, GraphQLError>;

#[derive(Debug, Clone)]
pub enum GraphQLError {
    #[allow(dead_code)]
    ItemNotFoundError,
    MissingDataError,
    NetworkError(Arc<reqwest::Error>),
    ResponseError(Vec<graphql_client::Error>),
    ServerError,
    #[allow(dead_code)]
    WrongDataTypeError,
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            GraphQLError::ItemNotFoundError => write!(f, "Unable to find item"),
            GraphQLError::MissingDataError => write!(
                f,
                "GraphQL response did not error, but does not have required data"
            ),
            GraphQLError::NetworkError(_) => write!(f, "Network error performing GraphQL query"),
            GraphQLError::ResponseError(_) => write!(
                f,
                "GraphQL call successfully executed, but the response has errors"
            ),
            GraphQLError::ServerError => write!(f, "General server error"),
            GraphQLError::WrongDataTypeError => write!(f, "Wrong GraphQL type returned"),
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
