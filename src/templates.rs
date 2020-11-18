use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use graphql_client::*;

pub struct Templates {}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/template_queries.graphql",
    response_derives = "Debug"
)]
pub struct GetTemplateByNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/template_queries.graphql",
    response_derives = "Debug"
)]
pub struct TemplatesQuery;

impl Templates {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_body_by_name(
        &self,
        environment_name: Option<&str>,
        template_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let query = GetTemplateByNameQuery::build_query(get_template_by_name_query::Variables {
            environment_name: environment_name.map(|name| name.to_string()),
            template_name: template_name.to_string(),
        });
        let response_body = graphql_request::<_, get_template_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
                .template
                .and_then(|t| t.evaluated))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_templates(&self) -> GraphQLResult<Vec<String>> {
        let query = TemplatesQuery::build_query(templates_query::Variables {});
        let response_body = graphql_request::<_, templates_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            let mut list = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .templates
                .nodes
                .into_iter()
                .map(|n| n.name)
                .collect::<Vec<String>>();
            list.sort();

            Ok(list)
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
