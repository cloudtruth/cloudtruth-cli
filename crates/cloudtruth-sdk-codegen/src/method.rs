use color_eyre::Result;
use indexmap::IndexMap;
use openapiv3::{Operation, Parameter, RequestBody, Responses};

use uritemplate::UriTemplate;

pub struct SdkOperation {
    pub path_template: UriTemplate,
    pub http_method: http::Method,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub request_body: Option<RequestBody>,
    pub parameters: Vec<Parameter>,
    pub responses: Responses,
    pub security: Option<Vec<IndexMap<String, Vec<String>>>>,
}

impl SdkOperation {
    pub fn from_openapi_operation(path: &str, method: &str, op: Operation) -> Result<SdkOperation> {
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

        Ok(SdkOperation {
            path_template: UriTemplate::new(path),
            http_method: method.parse()?,
            description,
            summary,
            operation_id,
            tags,
            deprecated,
            request_body,
            parameters,
            responses,
            security,
        })
    }
}
