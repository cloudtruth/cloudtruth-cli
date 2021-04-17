use crate::config::DEFAULT_PROJ_NAME;
use crate::graphql::prelude::graphql_request;
use crate::graphql::{GraphQLError, GraphQLResult};
use get_implicit_template_query::ImplicitTemplateEnum;
use graphql_client::*;
use std::str::FromStr;

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
pub struct GetImplicitTemplateQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/template_queries.graphql",
    response_derives = "Debug"
)]
pub struct TemplatesQuery;

pub struct TemplateDetails {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl Templates {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_body_by_name(
        &self,
        organization_id: Option<&str>,
        project_name: Option<String>,
        environment_name: Option<&str>,
        template_name: &str,
    ) -> GraphQLResult<Option<String>> {
        let query = GetTemplateByNameQuery::build_query(get_template_by_name_query::Variables {
            organization_id: organization_id.map(|id| id.to_string()),
            project_name: project_name.clone(),
            environment_name: environment_name.map(|name| name.to_string()),
            template_name: template_name.to_string(),
        });
        let response_body = graphql_request::<_, get_template_by_name_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                Ok(project.template.and_then(|t| t.evaluated))
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    project_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_body_by_implicit_name(
        &self,
        organization_id: Option<&str>,
        environment_name: Option<&str>,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
        contains: Option<&str>,
        export: bool,
        secrets: bool,
        template_format: &str,
    ) -> GraphQLResult<Option<String>> {
        impl FromStr for ImplicitTemplateEnum {
            type Err = ();

            fn from_str(input: &str) -> Result<ImplicitTemplateEnum, Self::Err> {
                match input {
                    "docker" => Ok(ImplicitTemplateEnum::DOCKER),
                    "dotenv" => Ok(ImplicitTemplateEnum::DOTENV),
                    "shell" => Ok(ImplicitTemplateEnum::SHELL),
                    _ => Err(()),
                }
            }
        }

        let format: ImplicitTemplateEnum = template_format.parse().unwrap();
        let query = GetImplicitTemplateQuery::build_query(get_implicit_template_query::Variables {
            organization_id: organization_id.map(|id| id.to_string()),
            environment_name: environment_name.map(|name| name.to_string()),
            template_format: format,
            filters: get_implicit_template_query::ImplicitTemplateFilters {
                starts_with: starts_with.map(|search| search.to_string()),
                ends_with: ends_with.map(|search| search.to_string()),
                contains: contains.map(|search| search.to_string()),
                secrets: Some(secrets),
                export: Some(export),
            },
        });
        let response_body =
            graphql_request::<_, get_implicit_template_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            Ok(data
                .viewer
                .organization
                .expect("Primary organization not found")
                .implicit_template
                .and_then(|t| t.evaluated))
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }

    pub fn get_template_details(
        &self,
        organization_id: Option<&str>,
        project_name: Option<String>,
    ) -> GraphQLResult<Vec<TemplateDetails>> {
        let query = TemplatesQuery::build_query(templates_query::Variables {
            organization_id: organization_id.map(|id| id.to_string()),
            project_name: project_name.clone(),
        });
        let response_body = graphql_request::<_, templates_query::ResponseData>(&query)?;

        if let Some(errors) = response_body.errors {
            Err(GraphQLError::ResponseError(errors))
        } else if let Some(data) = response_body.data {
            if let Some(project) = data
                .viewer
                .organization
                .expect("Primary organization not found")
                .project
            {
                let mut list: Vec<TemplateDetails> = project
                    .templates
                    .nodes
                    .into_iter()
                    .map(|n| TemplateDetails {
                        id: n.id,
                        name: n.name,
                        description: n.description.unwrap_or_default(),
                    })
                    .collect();
                list.sort_by(|l, r| l.name.cmp(&r.name));
                Ok(list)
            } else {
                Err(GraphQLError::ProjectNotFoundError(
                    project_name.unwrap_or_else(|| DEFAULT_PROJ_NAME.to_string()),
                ))
            }
        } else {
            Err(GraphQLError::MissingDataError)
        }
    }
}
