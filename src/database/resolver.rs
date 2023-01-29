use crate::database::{Environments, OpenApiConfig, Projects, ResolveError, ResolvedDetails};

pub struct Resolver {}

impl Resolver {
    pub fn new() -> Self {
        Self {}
    }

    /// Resolves the environment and project strings.
    ///
    /// If either fails, it prints an error and exits.
    /// On success, it returns a `ResolvedDetails` structure that contains ids to avoid needing to resolve
    /// the names again.
    pub fn resolve_ids(
        &self,
        rest_cfg: &OpenApiConfig,
        proj_name: &str,
        env_name: &str,
    ) -> Result<ResolvedDetails, ResolveError> {
        // The `err` value is used to allow accumulation of multiple errors to the user.
        let mut errors: Vec<String> = vec![];
        let environments = Environments::new();
        let env_id = environments.get_id(rest_cfg, env_name)?;
        if env_id.is_none() {
            errors.push(format!(
                "The '{env_name}' environment could not be found in your account.",
            ));
        }

        let mut proj_id: Option<String> = None;
        if !proj_name.is_empty() {
            let projects = Projects::new();
            proj_id = projects.get_id(rest_cfg, proj_name)?;
            if proj_id.is_none() {
                errors.push(format!(
                    "The '{proj_name}' project could not be found in your account.",
                ));
            }
        } else {
            errors.push("No project name was provided!".to_owned());
        }

        // if any errors were encountered, exit with an error code
        if !errors.is_empty() {
            Err(ResolveError::ResolutionNotFound(errors))
        } else {
            Ok(ResolvedDetails::new(
                env_name.to_string(),
                env_id.unwrap(),
                proj_name.to_string(),
                proj_id.unwrap(),
            ))
        }
    }
}
