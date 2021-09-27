use crate::database::{
    extract_details, generic_response_message, OpenApiConfig, TemplateDetails, TemplateHistory,
    PAGE_SIZE,
};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    PatchedTemplate, TemplateCreate, TemplateLookupError, TemplatePreview,
};
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::result::Result;

pub struct Templates {}

#[derive(Debug)]
pub enum TemplateError {
    AuthError(String),
    CreateApi(Error<ProjectsTemplatesCreateError>),
    EvaluateFailed(TemplateLookupError),
    ListError(Error<ProjectsTemplatesListError>),
    PreviewApi(Error<ProjectsTemplatePreviewCreateError>),
    RetrieveApi(Error<ProjectsTemplatesRetrieveError>),
    UpdateApi(Error<ProjectsTemplatesPartialUpdateError>),
    ResponseError(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::AuthError(msg) => write!(f, "Not Authenticated: {}", msg),
            TemplateError::ResponseError(msg) => write!(f, "{}", msg),
            TemplateError::EvaluateFailed(tle) => {
                write!(f, "Evaluation failed:{}", template_eval_errors(tle))
            }
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for TemplateError {}

pub fn template_eval_errors(tle: &TemplateLookupError) -> String {
    let mut failures: Vec<String> = tle
        .detail
        .iter()
        .map(|e| format!("{}: {}", e.parameter_name, e.error_detail))
        .collect();
    if failures.is_empty() {
        failures.push("No details available".to_string());
    }
    let prefix = "\n  ";
    format!("{}{}", prefix, failures.join(prefix))
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
                    _ => Err(TemplateError::ResponseError(generic_response_message(
                        &content.status,
                        &content.content,
                    ))),
                },
                Err(e) => Err(TemplateError::RetrieveApi(e)),
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
                401 => Err(TemplateError::AuthError(extract_details(&content.content))),
                403 => Err(TemplateError::AuthError(extract_details(&content.content))),
                _ => Err(TemplateError::ResponseError(generic_response_message(
                    &content.status,
                    &content.content,
                ))),
            },
            Err(e) => Err(TemplateError::ListError(e)),
        }
    }

    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_name: &str,
    ) -> Result<Option<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        let response =
            projects_templates_list(rest_cfg, proj_id, Some(template_name), None, PAGE_SIZE)?;

        if let Some(templates) = response.results {
            if templates.is_empty() {
                Ok(None)
            } else {
                // TODO: handle more than one?
                let template = &templates[0];
                Ok(Some(TemplateDetails::from(template)))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_template_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
    ) -> Result<Vec<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        let response = projects_templates_list(rest_cfg, proj_id, None, None, PAGE_SIZE)?;
        let mut list: Vec<TemplateDetails> = Vec::new();

        if let Some(templates) = response.results {
            for template in templates {
                list.push(TemplateDetails::from(&template));
            }
            list.sort_by(|l, r| l.name.cmp(&r.name));
        }
        Ok(list)
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
                _ => Err(TemplateError::ResponseError(generic_response_message(
                    &content.status,
                    &content.content,
                ))),
            },
            Err(e) => Err(TemplateError::CreateApi(e)),
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
                _ => Err(TemplateError::ResponseError(generic_response_message(
                    &content.status,
                    &content.content,
                ))),
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
                _ => Err(TemplateError::ResponseError(generic_response_message(
                    &content.status,
                    &content.content,
                ))),
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
