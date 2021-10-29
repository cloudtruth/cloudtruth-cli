use crate::database::{
    auth_details, response_message, OpenApiConfig, TemplateDetails, TemplateError, TemplateHistory,
    Users, PAGE_SIZE,
};
use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{PatchedTemplate, TemplateCreate, TemplatePreview};
use std::result::Result;

const NO_ORDERING: Option<&str> = None;

pub struct Templates {}

fn response_error(
    status: &reqwest::StatusCode,
    content: &str,
    env_name: Option<&str>,
) -> TemplateError {
    match status.as_u16() {
        401 => auth_error(content),
        403 => auth_error(content),
        404 => {
            let msg = response_message(status, content);
            if msg.contains("No Environment matches") && env_name.is_some() {
                TemplateError::EnvironmentMissing(
                    env_name.unwrap_or_default().to_string(),
                    "".to_string(),
                )
            } else if msg.contains("No HistoricalEnvironment matches") {
                TemplateError::EnvironmentMissing(
                    env_name.unwrap_or_default().to_string(),
                    " at specified time/tag".to_string(),
                )
            } else {
                TemplateError::ResponseError(msg)
            }
        }
        _ => TemplateError::ResponseError(response_message(status, content)),
    }
}

fn auth_error(content: &str) -> TemplateError {
    TemplateError::Authentication(auth_details(content))
}

impl Templates {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_id(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        proj_id: &str,
        template_name: &str,
    ) -> Result<String, TemplateError> {
        let details = self.get_details_by_name(
            rest_cfg,
            proj_name,
            proj_id,
            template_name,
            false,
            false,
            None,
            None,
            None,
        )?;
        Ok(details.id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_details_by_name(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        proj_id: &str,
        template_name: &str,
        evaluate: bool,
        show_secrets: bool,
        env_name: Option<String>,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<TemplateDetails, TemplateError> {
        let response = projects_templates_list(
            rest_cfg,
            proj_id,
            as_of,
            env_name.as_deref(),
            Some(evaluate),
            Some(!show_secrets),
            Some(template_name),
            NO_ORDERING,
            None,
            PAGE_SIZE,
            tag.as_deref(),
        );
        match response {
            Ok(data) => match data.results {
                Some(templates) => {
                    if templates.is_empty() {
                        Err(TemplateError::NotFound(
                            template_name.to_string(),
                            proj_name.to_string(),
                        ))
                    } else {
                        let template = &templates[0];
                        Ok(TemplateDetails::from(template))
                    }
                }
                _ => Err(TemplateError::NotFound(
                    template_name.to_string(),
                    proj_name.to_string(),
                )),
            },
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatesListError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(
                    &content.status,
                    &content.content,
                    env_name.as_deref(),
                )),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn get_template_details(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
    ) -> Result<Vec<TemplateDetails>, TemplateError> {
        let evaluate = Some(false);
        let mask_secrets = Some(true);
        let env_name = None;
        let as_of = None;
        let tag = None;
        let name = None; // get everything
        let response = projects_templates_list(
            rest_cfg,
            proj_id,
            as_of,
            env_name,
            evaluate,
            mask_secrets,
            name,
            NO_ORDERING,
            None,
            PAGE_SIZE,
            tag,
        );
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
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatesListError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content, env_name)),
            },
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
                _ => Err(response_error(&content.status, &content.content, None)),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    pub fn delete_template(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_id: &str,
        template_id: &str,
    ) -> Result<(), TemplateError> {
        let response = projects_templates_destroy(rest_cfg, template_id, proj_id);
        match response {
            Ok(_) => Ok(()),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content, None))
            }
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
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
            has_secret: None,
            referenced_parameters: None,
            referenced_templates: None,
            referencing_templates: None,
            referencing_values: None,
            created_at: None,
            modified_at: None,
            evaluated: None,
        };
        let response =
            projects_templates_partial_update(rest_cfg, template_id, project_id, Some(template));
        match response {
            Ok(r) => Ok(Some(r.id)),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatesPartialUpdateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(&content.status, &content.content, None)),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn preview_template(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        env_name: &str,
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
            Some(env_name),
            Some(!show_secrets),
            tag.as_deref(),
        );
        match response {
            Ok(r) => Ok(r.body),
            Err(ResponseError(ref content)) => match &content.entity {
                Some(ProjectsTemplatePreviewCreateError::Status422(tle)) => {
                    Err(TemplateError::EvaluateFailed(tle.clone()))
                }
                _ => Err(response_error(
                    &content.status,
                    &content.content,
                    Some(env_name),
                )),
            },
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    fn resolve_user_ids(&self, rest_cfg: &OpenApiConfig, histories: &mut [TemplateHistory]) {
        if !histories.is_empty() {
            let users = Users::new();
            let user_map = users.get_user_id_to_name_map(rest_cfg);
            if let Ok(user_map) = user_map {
                let default_username = "".to_string();
                for entry in histories {
                    entry.user_name = user_map
                        .get(&entry.user_id)
                        .unwrap_or(&default_username)
                        .clone();
                }
            }
        }
    }

    /// Gets the template history for all templates in the project.
    pub fn get_histories(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        _env_name_or_id: &str,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Vec<TemplateHistory>, TemplateError> {
        let response =
            projects_templates_timelines_retrieve(rest_cfg, project_id, as_of, tag.as_deref());
        match response {
            Ok(data) => {
                let mut histories: Vec<TemplateHistory> =
                    data.results.iter().map(TemplateHistory::from).collect();
                self.resolve_user_ids(rest_cfg, &mut histories);
                Ok(histories)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content, None))
            }
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }

    /// Gets the template history for a single template in the project.
    pub fn get_history_for(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        template_id: &str,
        _env_name_name_or_id: &str,
        as_of: Option<String>,
        tag: Option<String>,
    ) -> Result<Vec<TemplateHistory>, TemplateError> {
        let response = projects_templates_timeline_retrieve(
            rest_cfg,
            template_id,
            project_id,
            as_of,
            tag.as_deref(),
        );
        match response {
            Ok(data) => {
                let mut histories: Vec<TemplateHistory> =
                    data.results.iter().map(TemplateHistory::from).collect();
                self.resolve_user_ids(rest_cfg, &mut histories);
                Ok(histories)
            }
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content, None))
            }
            Err(e) => Err(TemplateError::UnhandledError(e.to_string())),
        }
    }
}
