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
        TemplateDetails {
            id: api_temp.id.clone(),
            name: api_temp.name.clone(),
            description: api_temp.description.clone().unwrap_or_default(),
        }
    }
}

impl Templates {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_body_by_name(
        &self,
        proj_id: &str,
        env_id: &str,
        template_name: &str,
        show_secrets: bool,
    ) -> Result<Option<String>, Error<ProjectsTemplatesRetrieveError>> {
        // TODO: convert template name to template id outside??
        let response = self.get_details_by_name(proj_id, template_name);

        if let Ok(Some(details)) = response {
            let rest_cfg = open_api_config();
            let response = projects_templates_retrieve(
                &rest_cfg,
                &details.id,
                proj_id,
                Some(env_id),
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
        proj_id: &str,
        template_name: &str,
    ) -> Result<Option<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        let rest_cfg = open_api_config();
        let response = projects_templates_list(&rest_cfg, proj_id, Some(template_name), None)?;

        if let Some(templates) = response.results {
            if template_name.is_empty() {
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
        proj_id: &str,
    ) -> Result<Vec<TemplateDetails>, Error<ProjectsTemplatesListError>> {
        let rest_cfg = open_api_config();
        let response = projects_templates_list(&rest_cfg, proj_id, None, None)?;
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
