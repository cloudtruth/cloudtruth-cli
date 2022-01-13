use crate::database::{response_message, BackupError, OpenApiConfig};
use cloudtruth_restapi::apis::backup_api::*;
use cloudtruth_restapi::apis::Error::ResponseError;
use cloudtruth_restapi::models::BackupDataSnapshot;
use std::result::Result;

// This is a quick hack -- no need to re-invent the wheel here. Since this is serializable, use it.
pub type BackupSnapshotDetails = BackupDataSnapshot;

pub struct Backups {}

fn response_error(status: &reqwest::StatusCode, content: &str) -> BackupError {
    BackupError::ResponseError(response_message(status, content))
}

impl Backups {
    pub fn new() -> Self {
        Self {}
    }

    pub fn data_snapshot(
        &self,
        rest_cfg: &OpenApiConfig,
    ) -> Result<BackupSnapshotDetails, BackupError> {
        let response = backup_snapshot_create(rest_cfg);
        match response {
            Ok(snapshot) => Ok(snapshot),
            Err(ResponseError(ref content)) => {
                Err(response_error(&content.status, &content.content))
            }
            Err(e) => Err(BackupError::UnhandledError(e.to_string())),
        }
    }
}
