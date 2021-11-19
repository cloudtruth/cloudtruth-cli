use crate::database::{
    auth_details, page_size, response_message, AuditLogDetails, AuditLogError, AuditLogSummary,
    OpenApiConfig,
};
use cloudtruth_restapi::apis::audit_api::{audit_list, audit_summary_retrieve};
use cloudtruth_restapi::apis::Error::ResponseError;

const NO_ORDERING: Option<&str> = None;

pub struct AuditLogs {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> AuditLogError {
    match status.as_u16() {
        401 => AuditLogError::Authentication(auth_details(content)),
        403 => AuditLogError::Authentication(auth_details(content)),
        _ => AuditLogError::ResponseError(response_message(status, content)),
    }
}

impl AuditLogs {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_audit_log_details(
        &self,
        rest_cfg: &OpenApiConfig,
        object_type: Option<&str>,
        action: Option<&str>,
        name_contains: Option<&str>,
        max_entries: usize,
        before: Option<String>,
        after: Option<String>,
        user_id: Option<&str>,
    ) -> Result<Vec<AuditLogDetails>, AuditLogError> {
        let mut total_details: Vec<AuditLogDetails> = vec![];
        let mut page_count = 1;
        loop {
            let response = audit_list(
                rest_cfg,
                action,
                after.clone(),
                before.clone(),
                None,
                object_type,
                NO_ORDERING,
                Some(page_count),
                page_size(rest_cfg),
                user_id,
            );
            match response {
                Ok(data) => match data.results {
                    Some(list) => {
                        if list.is_empty() {
                            break;
                        } else {
                            let mut current: Vec<AuditLogDetails> =
                                list.iter().map(AuditLogDetails::from).collect();
                            if let Some(contains) = name_contains {
                                current.retain(|d| d.object_name.contains(contains));
                            }
                            total_details.extend(current);
                            page_count += 1;
                            if data.next.is_none() {
                                break;
                            }
                            if max_entries != 0 && total_details.len() >= max_entries {
                                break;
                            }
                        }
                    }
                    None => {
                        break;
                    }
                },
                Err(ResponseError(ref content)) => {
                    return Err(response_error(&content.status, &content.content));
                }
                Err(e) => {
                    return Err(AuditLogError::UnhandledError(e.to_string()));
                }
            }
        }
        if max_entries != 0 && total_details.len() > max_entries {
            total_details.truncate(max_entries);
        }
        Ok(total_details)
    }

    pub fn get_audit_log_summary(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<AuditLogSummary, AuditLogError> {
        let response = audit_summary_retrieve(rest_cfg);
        match response {
            Ok(ref api) => Ok(AuditLogSummary::from(api)),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(AuditLogError::UnhandledError(e.to_string())),
        }
    }
}
