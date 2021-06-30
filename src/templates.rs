use crate::openapi::open_api_config;

use cloudtruth_restapi::apis::projects_api::*;
use cloudtruth_restapi::apis::Error;
use cloudtruth_restapi::models::Template;

pub struct Templates {}

pub struct TemplateDetails {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl From<&Template> for TemplateDetails {
    fn from(api_temp: &Template) -> Self {
        let description = api_temp.description.clone();
        TemplateDetails {
            id: api_temp.id.clone(),
            name: api_temp.name.clone(),
            description: description.unwrap_or_default(),
        }
    }
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
        show_secrets: bool,
    ) -> Result<Option<String>, Error<ProjectsTemplatesRetrieveError>> {
        // TODO: convert template name to template id outside??
        // TODO: project name or id? environment name or id?
        let response =
            self.get_details_by_name(organization_id, project_name.clone(), Some(template_name));

        if let Ok(Some(details)) = response {
            let rest_cfg = open_api_config();
            let response = projects_templates_retrieve(
                &rest_cfg,
                &details.id,
                project_name.as_deref().unwrap_or_default(),
                environment_name,
                Some(show_secrets),
            )?;
            Ok(response.body)
        } else {
            // TODO: handle template not found??
            Ok(None)
        }
    }

    pub fn get_details_by_name(
        &self,
        _organization_id: Option<&str>,
        project_name: Option<String>,
        template_name: Option<&str>,
    ) -> Result<Option<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        // TODO: project name or id?
        let rest_cfg = open_api_config();
        let response = projects_templates_list(
            &rest_cfg,
            project_name.unwrap_or_default().as_str(),
            template_name,
            None,
        )?;

        if let Some(templates) = response.results {
            // TODO: handle more than one?
            let template = &templates[0];
            Ok(Some(TemplateDetails::from(template)))
        } else {
            Ok(None)
        }
    }

    pub fn get_template_details(
        &self,
        _organization_id: Option<&str>,
        project_name: Option<String>,
    ) -> Result<Vec<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        // TODO: project name or id?
        let rest_cfg = open_api_config();
        let response = projects_templates_list(
            &rest_cfg,
            project_name.unwrap_or_default().as_str(),
            None,
            None,
        )?;
        let mut list: Vec<TemplateDetails> = Vec::new();

        if let Some(templates) = response.results {
            for template in templates {
                list.push(TemplateDetails::from(&template));
            }
            list.sort_by(|l, r| l.name.cmp(&r.name));
        }
        Ok(list)
    }
}
