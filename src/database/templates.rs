use crate::database::{
    auth_details, response_message, OpenApiConfig, TemplateDetails, TemplateError, TemplateHistory,
    PAGE_SIZE,
};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{PatchedTemplate, TemplateCreate, TemplatePreview};
use std::result::Result;

pub struct Templates {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> TemplateError {
    TemplateError::ResponseError(response_message(status, content))
}

fn auth_error(content: &str) -> TemplateError {
    TemplateError::Authentication(auth_details(content))
}

impl Templates {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_body_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        env_id: &str,
        template_name: &str,
        show_secrets: bool,
    ) -> Result<Option<String>, TemplateError> {
        // TODO: convert template name to template id outside??
        let response = self.get_details_by_name(rest_cfg, proj_id, template_name);

        if let Ok(Some(details)) = response {
            let response = projects_templates_retrieve(
                rest_cfg,
                &details.id,
                proj_id,
                Some(env_id),
                Some(!show_secrets),
            );
            match response {
                Ok(r) => Ok(r.body),
                Err(ResponseError(ref content)) => match &content.entity {
                    Some(ProjectsTemplatesRetrieveError::Status422(tle)) => {
                        Err(TemplateError::EvaluateFailed(tle.clone()))
                    }
                    _ => Err(response_error(&content.status, &content.content)),
                },
                Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_unevaluated_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_name: &str,
    ) -> Result<Option<TemplateDetails>, TemplateError> {
        // Currently, the only way to get the unevaluated body is to list the templates.
        let response =
            projects_templates_list(rest_cfg, proj_id, Some(template_name), None, PAGE_SIZE);

        match response {
            Ok(data) => match data.results {
                Some(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one??
                        let template = &list[0];
                        Ok(Some(TemplateDetails::from(template)))
                    }
                }
                _ => Ok(None),
            },
            Err(ResponseError(ref content)) => match content.status.as_u16() {
                401 => Err(auth_error(&content.content)),
                403 => Err(auth_error(&content.content)),
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_name: &str,
    ) -> Result<Option<TemplateDetails>, TemplateError> {
        let response =
            projects_templates_list(rest_cfg, proj_id, Some(template_name), None, PAGE_SIZE);
        match response {
            Ok(data) => match data.results {
                Some(templates) => {
                    if templates.is_empty() {
                        Ok(None)
                    } else {
                        // TODO: handle more than one?
                        let template = &templates[0];
                        Ok(Some(TemplateDetails::from(template)))
                    }
                }
                None => Ok(None),
            },
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_template_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
    ) -> Result<Vec<TemplateDetails>, TemplateError> {
        let response = projects_templates_list(rest_cfg, proj_id, None, None, PAGE_SIZE);
        match response {
            Ok(data) => {
                let mut list: Vec<TemplateDetails> = Vec::new();
                if let Some(templates) = data.results {
                    for template in templates {
                        list.push(TemplateDetails::from(&template));
                    }
                    list.sort_by(|l, r| l.name.cmp(&r.name));
                }
                Ok(list)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn create_template(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_name: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<Option<String>, TemplateError> {
        let template_create = TemplateCreate {
            name: template_name.to_string(),
            description: description.map(String::from),
            body: Some(body.to_string()),
        };
        let response = projects_templates_create(rest_cfg, proj_id, template_create);
        match response {
            Ok(r) => Ok(Some(r.id)),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatesCreateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_template(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_id: &str,
    ) -> Result<(), Error<ProjectsTemplatesDestroyError>> {
        projects_templates_destroy(rest_cfg, template_id, proj_id)
    }

    pub fn update_template(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        template_id: &str,
        template_name: &str,
        description: Option<&str>,
        body: Option<&str>,
    ) -> Result<Option<String>, TemplateError> {
        let template = PatchedTemplate {
            url: None,
            id: None,
            name: Some(template_name.to_string()),
            description: description.map(String::from),
            body: body.map(String::from),
            parameters: None,
            references: None,
            referenced_by: None,
            has_secret: None,
            created_at: None,
            modified_at: None,
        };
        let response =
            projects_templates_partial_update(rest_cfg, template_id, project_id, Some(template));
        match response {
            Ok(r) => Ok(Some(r.id)),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatesPartialUpdateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(TemplateError::UpdateApi(e)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn preview_template(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        env_id: &str,
        body: &str,
        show_secrets: bool,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<String, TemplateError> {
        let preview = TemplatePreview {
            body: body.to_string(),
        };
        let response = projects_template_preview_create(
            rest_cfg,
            project_id,
            preview,
            as_of,
            Some(env_id),
            Some(!show_secrets),
            tag.as_deref(),
        );
        match response {
            Ok(r) => Ok(r.body),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatePreviewCreateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content)),
            },
            Err(e) => Err(TemplateError::PreviewApi(e)),
        }
    }

    /// Gets the template history for all templates in the project.
    pub fn get_histories(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Vec<TemplateHistory>, Error<ProjectsTemplatesTimelinesRetrieveError>> {
        let response =
            projects_templates_timelines_retrieve(rest_cfg, project_id, as_of, tag.as_deref())?;
        Ok(response.results.iter().map(TemplateHistory::from).collect())
    }

    /// Gets the template history for a single template in the project.
    pub fn get_history_for(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        template_id: &str,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Vec<TemplateHistory>, Error<ProjectsTemplatesTimelineRetrieveError>> {
        let response = projects_templates_timeline_retrieve(
            rest_cfg,
            template_id,
            project_id,
            as_of,
            tag.as_deref(),
        )?;
        Ok(response.results.iter().map(TemplateHistory::from).collect())
    }
}
