use crate::database::openapi::{extract_details, OpenApiConfig, PAGE_SIZE};

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::{PatchedTemplate, Template, TemplateCreate, TemplatePreview};
use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::result::Result;

const NO_DETAILS_ERR: &str = "No details available";

pub struct Templates {}

#[derive(Debug)]
pub struct TemplateDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
}

impl From<&Template> for TemplateDetails {
    fn from(api_temp: &Template) -> Self {
        TemplateDetails {
            id: api_temp.id.clone(),
            name: api_temp.name.clone(),
            description: api_temp.description.clone().unwrap_or_default(),
            body: api_temp.body.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub enum TemplateError {
    AuthError(String),
    ListError(Error<ProjectsTemplatesListError>),
    RetrieveApi(Error<ProjectsTemplatesRetrieveError>),
    EvaluateApi(Error<ProjectsTemplatePreviewCreateError>),
    EvaluateFailed(Vec<String>),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::AuthError(msg) => write!(f, "Not Authenticated: {}", msg),
            TemplateError::ListError(e) => write!(f, "{}", e.to_string()),
            TemplateError::RetrieveApi(e) => write!(f, "{}", e.to_string()),
            TemplateError::EvaluateApi(e) => write!(f, "{}", e.to_string()),
            TemplateError::EvaluateFailed(reasons) => {
                write!(f, "Evaluation failed:\n  {}", reasons.join("\n  "))
            }
        }
    }
}

impl error::Error for TemplateError {}

fn unquote(orig: &str) -> String {
    orig.trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

fn extract_reasons(content: &str) -> Vec<String> {
    let mut reasons: Vec<String> = vec![];
    let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(content);
    if let Ok(data) = json_result {
        if let Some(details) = data.get("detail") {
            if details.is_string() {
                reasons.push(unquote(&details.to_string()))
            } else if details.is_array() {
                for failure in details.as_array().unwrap() {
                    if failure.is_string() {
                        reasons.push(unquote(&failure.to_string()))
                    } else if failure.is_object() {
                        if let Some(name) = failure.get("parameter_name") {
                            if let Some(error) = failure.get("error_detail") {
                                reasons.push(format!(
                                    "{}: {}",
                                    unquote(&name.to_string()),
                                    unquote(&error.to_string())
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if reasons.is_empty() {
        reasons.push(NO_DETAILS_ERR.to_string())
    }
    reasons
}

fn evaluation_error(content: &str) -> TemplateError {
    TemplateError::EvaluateFailed(extract_reasons(content))
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
                Err(ResponseError(r)) => Err(evaluation_error(&r.content)),
                Err(e) => Err(TemplateError::RetrieveApi(e)),
            }
        } else {
            // TODO: handle template not found??
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
    ) -> Result<Option<String>, Error<ProjectsTemplatesCreateError>> {
        let template_create = TemplateCreate {
            name: template_name.to_string(),
            description: description.map(String::from),
            body: Some(body.to_string()),
        };
        let response = projects_templates_create(rest_cfg, proj_id, template_create)?;
        // TODO: `TemplateCreate` does not have an id, so return the name of the newly minted template
        Ok(Some(response.name))
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
    ) -> Result<Option<String>, Error<ProjectsTemplatesPartialUpdateError>> {
        let template = PatchedTemplate {
            url: None,
            id: None,
            name: Some(template_name.to_string()),
            description: description.map(String::from),
            body: body.map(String::from),
            parameters: None,
            has_secret: None,
            created_at: None,
            modified_at: None,
        };
        let response =
            projects_templates_partial_update(rest_cfg, template_id, project_id, Some(template))?;
        Ok(Some(response.id))
    }

    pub fn preview_template(
        &self,
        rest_cfg: &OpenApiConfig,
        project_id: &str,
        env_id: &str,
        body: &str,
        show_secrets: bool,
    ) -> Result<String, TemplateError> {
        let preview = TemplatePreview {
            body: body.to_string(),
        };
        let response = projects_template_preview_create(
            rest_cfg,
            project_id,
            preview,
            Some(env_id),
            Some(!show_secrets),
        );
        match response {
            Ok(r) => Ok(r.body),
            Err(ResponseError(r)) => Err(evaluation_error(&r.content)),
            Err(e) => Err(TemplateError::EvaluateApi(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_errors_empty() {
        let values = vec![
            "",
            "{}",
            "{\"detail\":[]}",
            "{\"detail\":[{}]}",
            "{\"detail\":[{\"error_code\": \"foo\"}]}",
            "{\"detail\":[{\"parameter_name\":\"pname\"}]}",
        ];
        for content in values {
            let results = extract_reasons(content);
            assert_eq!(1, results.len());
            let value = &results[0];
            assert_eq!(value.as_str(), NO_DETAILS_ERR);
        }
    }

    #[test]
    fn parse_errors_single_string() {
        let content = "{\"detail\":\"sample string\"}";
        let results = extract_reasons(content);
        assert_eq!(1, results.len());
        let value = &results[0];
        assert_eq!(value.as_str(), "sample string");
    }

    #[test]
    fn parse_errors_multiple_string() {
        let content = "{\"detail\":[\"sample string\",\"another string\"]}";
        let results = extract_reasons(content);
        assert_eq!(2, results.len());
        let value = &results[0];
        assert_eq!(value.as_str(), "sample string");
        let value = &results[1];
        assert_eq!(value.as_str(), "another string");
    }

    #[test]
    fn parse_errors_single_struct() {
        let content = "{\"detail\":[{\"error_code\":\"retrieval_error\",\"error_detail\":\"Error occurred while retrieving content from `test://foo/bar/`: unit test forced\",\"parameter_id\":\"c82b08af-6254-4967-85c1-66af5ffd9554\",\"parameter_name\":\"FriedRice\"}]}";
        let results = extract_reasons(content);
        assert_eq!(1, results.len());
        let value = &results[0];
        assert_eq!(value.as_str(), "FriedRice: Error occurred while retrieving content from `test://foo/bar/`: unit test forced");
    }

    #[test]
    fn parse_errors_multiple_struct() {
        let content = "{\"detail\":[{\"parameter_name\":\"pname\",\"error_detail\":\"some value\"},{\"parameter_name\":\"another\",\"error_detail\":\"detail two\"}]}";
        let results = extract_reasons(content);
        assert_eq!(2, results.len());
        let value = &results[0];
        assert_eq!(value.as_str(), "pname: some value");
        let value = &results[1];
        assert_eq!(value.as_str(), "another: detail two");
    }
}
