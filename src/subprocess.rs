use crate::config::{DEFAULT_ENV_NAME, ENV_VAR_PREFIX};
use crate::environments::Environments;
use crate::parameters::Parameters;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::env;
use subprocess::Exec;

// for improved readability
pub type EnvSettings = HashMap<String, String>;

pub trait SubProcessIntf {
    fn set_environment(
        &mut self,
        org_id: Option<&str>,
        env: Option<&str>,
        environments: &Environments,
        preserve: bool,
        overrides: &[String],
        removals: &[String],
    ) -> Result<()>;

    fn run_command(&self, command: &str, arguments: &[String]) -> Result<()>;
}

pub struct SubProcess {
    env_vars: EnvSettings,
}

impl SubProcess {
    pub fn new() -> Self {
        Self {
            env_vars: Default::default(),
        }
    }

    fn current_env(&self) -> EnvSettings {
        // Create a EnvSettings from the current set of environment variables (excluding a few).
        let exclude = ["PS1", "TERM", "HOME"];

        env::vars()
            .filter(|(ref k, _)| !exclude.contains(&k.as_str()))
            .collect()
    }

    fn get_ct_vars(
        &self,
        org_id: Option<&str>,
        env: Option<&str>,
        environments: &Environments,
    ) -> Result<EnvSettings> {
        // Create EnvSettings with all the CloudTruth environment values for this environment.
        let mut ct_vars = EnvSettings::new();
        let parameters = Parameters::new();
        let env_id = environments.get_id(org_id, env)?;
        let list = parameters.get_parameter_names(org_id, env_id)?;
        for key in list.iter() {
            let parameter = parameters.get_body(org_id, env, key)?;
            // Put the key/value pair into the environment
            let value = parameter.unwrap_or_else(|| "".to_string());
            ct_vars.insert(key.to_string(), value);
        }
        Ok(ct_vars)
    }

    fn process_overrides(&self, overrides: &[String]) -> EnvSettings {
        // Create EnvSettings with all the user-provided overrides.
        let mut over_vars = EnvSettings::new();
        for arg_val in overrides {
            let temp: Vec<&str> = arg_val.splitn(2, '=').collect();
            if temp.len() != 2 {
                // TODO: provide feedback to user
                //warn_user(format!("Ignoring {} due to  no '='", arg_val));
                continue;
            }
            over_vars.insert(temp[0].to_string(), temp[1].to_string());
        }
        over_vars
    }
}

impl SubProcessIntf for SubProcess {
    fn set_environment(
        &mut self,
        org_id: Option<&str>,
        env: Option<&str>,
        environments: &Environments,
        preserve: bool,
        overrides: &[String],
        removals: &[String],
    ) -> Result<()> {
        self.env_vars = if !preserve {
            EnvSettings::new()
        } else {
            self.current_env()
        };

        // Add breadcrumbs about which environment.
        self.env_vars.insert(
            format!("{}ENV", ENV_VAR_PREFIX),
            env.unwrap_or(DEFAULT_ENV_NAME).to_string(),
        );

        // Add in the items from the CloudTruth environment, and overrides.
        self.env_vars
            .extend(self.get_ct_vars(org_id, env, environments)?);
        self.env_vars.extend(self.process_overrides(overrides));

        // Remove the specified values.
        for r in removals {
            self.env_vars.remove(r.as_str());
        }

        Ok(())
    }

    fn run_command(&self, command: &str, arguments: &[String]) -> Result<()> {
        let mut sub_proc: Exec;

        if arguments.is_empty() {
            sub_proc = Exec::shell(command);
        } else {
            sub_proc = Exec::cmd(command).args(arguments);
        }

        // Common setup for the subprocess. By default, it streams stdin/stdout/stderr to parent.
        sub_proc = sub_proc.env_clear();
        for (key, value) in &self.env_vars {
            sub_proc = sub_proc.env(key, value);
        }

        sub_proc.join()?;
        Ok(())
    }
}
