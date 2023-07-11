use std::rc::Rc;

use color_eyre::{eyre::eyre, Result};
use indexmap::IndexMap;
use openapiv3::{Operation, Parameter, RequestBody, Responses};

use rfc6570_level_2::UriTemplate;

#[derive(Debug, Clone)]
pub struct ApiOperation {
    path_template: UriTemplate,
    http_method: http::Method,
    summary: Option<Rc<str>>,
    description: Option<Rc<str>>,
    operation_id: Option<Rc<str>>,
    tags: Vec<Rc<str>>,
    deprecated: bool,
    request_body: Option<RequestBody>,
    parameters: Vec<Parameter>,
    responses: Responses,
    security: Option<Vec<IndexMap<String, Vec<String>>>>,
}

impl ApiOperation {
    pub fn from_openapi(path: &str, method: &str, op: Operation) -> Result<ApiOperation> {
        let Operation {
            description,
            summary,
            operation_id,
            tags,
            request_body,
            parameters,
            responses,
            deprecated,
            security,
            ..
        } = op;
        let request_body = request_body.map(|b| b.into_item().unwrap());
        let parameters: Vec<Parameter> = parameters
            .into_iter()
            .map(|p| p.into_item().unwrap())
            .collect();

        Ok(ApiOperation {
            path_template: UriTemplate::new(path).map_err(|err| eyre!(Box::new(err)))?, // convert anyhow to eyre
            http_method: method.parse()?,
            description: description.map(|s| Rc::from(s.as_str())),
            summary: summary.map(|s| Rc::from(s.as_str())),
            operation_id: operation_id.map(|s| Rc::from(s.as_str())),
            tags: tags.into_iter().map(|s| Rc::from(s.as_str())).collect(),
            deprecated,
            request_body,
            parameters,
            responses,
            security,
        })
    }

    pub fn uri(&self) -> &str {
        self.path_template.uri()
    }

    pub fn http_method(&self) -> &http::Method {
        &self.http_method
    }

    pub fn summary(&self) -> Option<&Rc<str>> {
        self.summary.as_ref()
    }

    pub fn description(&self) -> Option<&Rc<str>> {
        self.description.as_ref()
    }

    pub fn operation_id(&self) -> Option<&Rc<str>> {
        self.operation_id.as_ref()
    }

    pub fn tags(&self) -> &[Rc<str>] {
        self.tags.as_ref()
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }
}
