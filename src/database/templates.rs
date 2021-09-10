use crate::database::openapi::{extract_details, OpenApiConfig, PAGE_SIZE};

use crate::database::HistoryAction;
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{
    PatchedTemplate, Template, TemplateCreate, TemplateLookupError, TemplatePreview,
    TemplateTimelineEntry,
};
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use std::result::Result;

pub struct Templates {}

#[derive(Debug)]
pub struct TemplateDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<&Template> for TemplateDetails {
    fn from(api_temp: &Template) -> Self {
        TemplateDetails {
            id: api_temp.id.clone(),
            name: api_temp.name.clone(),
            description: api_temp.description.clone().unwrap_or_default(),
            body: api_temp.body.clone().unwrap_or_default(),
            created_at: api_temp.created_at.clone(),
            modified_at: api_temp.modified_at.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateHistory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,

    // these are from the timeline
    pub date: String,
    pub change_type: HistoryAction,
    pub user: String, // may need name and the id
}

impl From<&TemplateTimelineEntry> for TemplateHistory {
    fn from(api: &TemplateTimelineEntry) -> Self {
        TemplateHistory {
            id: api.history_template.id.clone(),
            name: api.history_template.name.clone(),
            description: api.history_template.description.clone().unwrap_or_default(),
            body: api.history_template.body.clone().unwrap_or_default(),

            date: api.history_date.clone(),
            change_type: HistoryAction::from(*api.history_type.deref()),
            user: api.history_user.clone().unwrap_or_default(),
        }
    }
}

impl TemplateHistory {
    pub fn get_property(&self, name: &str) -> String {
        match name {
            "name" => self.name.clone(),
            "body" => self.body.clone(),
            "description" => self.description.clone(),
            x => format!("Unhandled property: {}", x),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_date(&self) -> String {
        self.date.clone()
    }

    pub fn get_action(&self) -> HistoryAction {
        self.change_type.clone()
    }
}

#[derive(Debug)]
pub enum TemplateError {
    AuthError(String),
    CreateApi(Error<ProjectsTemplatesCreateError>),
    EvaluateFailed(TemplateLookupError),
    ListError(Error<ProjectsTemplatesListError>),
    PreviewApi(Error<ProjectsTemplatePreviewCreateError>),
    RetrieveApi(Error<ProjectsTemplatesRetrieveError>),
    UpdateApi(Error<ProjectsTemplatesPartialUpdateError>),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::AuthError(msg) => write!(f, "Not Authenticated: {}", msg),
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
                    _ => Err(TemplateError::RetrieveApi(response.unwrap_err())),
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
                _ => Err(TemplateError::ListError(response.unwrap_err())),
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
                _ => Err(TemplateError::CreateApi(response.unwrap_err())),
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
                _ => Err(TemplateError::UpdateApi(response.unwrap_err())),
            },
            Err(e) => Err(TemplateError::UpdateApi(e)),
        }
    }

    pub fn preview_template(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        env_id: &str,
        body: &str,
        show_secrets: bool,
        as_of: Option<String>,
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
        );
        match response {
            Ok(r) => Ok(r.body),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatePreviewCreateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(TemplateError::PreviewApi(response.unwrap_err())),
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
    ) -> Result<Vec<TemplateHistory>, Error<ProjectsTemplatesTimelinesRetrieveError>> {
        let response = projects_templates_timelines_retrieve(rest_cfg, project_id, as_of)?;
        Ok(response.results.iter().map(TemplateHistory::from).collect())
    }

    /// Gets the template history for a single template in the project.
    pub fn get_history_for(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        template_id: &str,
        as_of: Option<String>,
    ) -> Result<Vec<TemplateHistory>, Error<ProjectsTemplatesTimelineRetrieveError>> {
        let response =
            projects_templates_timeline_retrieve(rest_cfg, template_id, project_id, as_of)?;
        Ok(response.results.iter().map(TemplateHistory::from).collect())
    }
}
