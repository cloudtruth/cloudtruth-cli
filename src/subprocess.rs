use crate::config::{CT_APP_REMOVABLE_VARS, CT_ENVIRONMENT, DEFAULT_ENV_NAME};
use crate::graphql::GraphQLError;
use crate::parameters::Parameters;
use crate::{warn_user, ResolvedIds};
use color_eyre::eyre::{ErrReport, Result};
use color_eyre::Report;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::{env, error};
use subprocess::{Exec, PopenError};

// for improved readability
pub type EnvSettings = HashMap<String, String>;

// NOTE: Hash and Debug are needed for testing... sigh
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Inheritance {
    None,
    Underlay,
    Overlay,
    Exclusive,
}

impl Display for Inheritance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl FromStr for Inheritance {
    type Err = ();

    fn from_str(input: &str) -> Result<Inheritance, Self::Err> {
        match input.to_lowercase().as_str() {
            "none" => Ok(Inheritance::None),
            "underlay" => Ok(Inheritance::Underlay),
            "overlay" => Ok(Inheritance::Overlay),
            "exclusive" => Ok(Inheritance::Exclusive),
            _ => Err(()),
        }
    }
}

pub type SubProcessResult<T> = std::result::Result<T, SubProcessError>;

#[derive(Clone, Debug)]
pub enum SubProcessError {
    EnvironmentCollisions(Vec<String>),
    GraphQLError(GraphQLError),
    ProcessRunError(String),
    ProcessOutputError(String),
}

impl error::Error for SubProcessError {}

impl fmt::Display for SubProcessError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            SubProcessError::EnvironmentCollisions(collisions) => {
                write!(
                    f,
                    "Conflicting definitions in run environment for: {}",
                    collisions.join(", ")
                )
            }
            SubProcessError::ProcessRunError(details) => {
                write!(f, "Process run error: {}", details)
            }
            SubProcessError::GraphQLError(err) => {
                write!(f, "Problem querying the server: {}", err.to_string())
            }
            SubProcessError::ProcessOutputError(details) => {
                write!(f, "Problem writing output: {}", details)
            }
        }
    }
}

impl From<GraphQLError> for SubProcessError {
    fn from(err: GraphQLError) -> Self {
        SubProcessError::GraphQLError(err)
    }
}

// NOTE: PopenError does not implement the Clone trait, so just pass through a String
impl From<PopenError> for SubProcessError {
    fn from(err: PopenError) -> Self {
        SubProcessError::ProcessRunError(format!("{}", err))
    }
}

// NOTE: ErrReport does not implement the Clone trait, so just pass through a String
impl From<ErrReport> for SubProcessError {
    fn from(err: Report) -> Self {
        SubProcessError::ProcessOutputError(format!("{}", err))
    }
}

pub trait SubProcessIntf {
    fn set_environment(
        &mut self,
        org_id: Option<&str>,
        resolved: &ResolvedIds,
        inherit: Inheritance,
        overrides: &[String],
        removals: &[String],
    ) -> SubProcessResult<()>;

    fn remove_ct_app_vars(&mut self);

    fn run_command(&self, command: &str, arguments: &[String]) -> SubProcessResult<()>;
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
        let exclude = ["PS1", "TERM"];

        env::vars()
            .filter(|(ref k, _)| !exclude.contains(&k.as_str()))
            .collect()
    }

    fn get_ct_vars(
        &self,
        org_id: Option<&str>,
        resolved: &ResolvedIds,
    ) -> SubProcessResult<EnvSettings> {
        // Create EnvSettings with all the CloudTruth environment values for this environment.
        let parameters = Parameters::new();
        let ct_vars = parameters.get_parameter_values(
            org_id,
            resolved.env_id.clone(),
            resolved.proj_name.clone(),
        )?;
        Ok(ct_vars)
    }

    fn process_overrides(&self, overrides: &[String]) -> Result<EnvSettings> {
        // Create EnvSettings with all the user-provided overrides.
        let mut over_vars = EnvSettings::new();
        for arg_val in overrides {
            let temp: Vec<&str> = arg_val.splitn(2, '=').collect();
            if temp.len() != 2 {
                warn_user(format!("Ignoring {} due to no '='", arg_val))?;
                continue;
            }
            over_vars.insert(temp[0].to_string(), temp[1].to_string());
        }
        Ok(over_vars)
    }
}

impl SubProcessIntf for SubProcess {
    fn set_environment(
        &mut self,
        org_id: Option<&str>,
        resolved: &ResolvedIds,
        inherit: Inheritance,
        overrides: &[String],
        removals: &[String],
    ) -> SubProcessResult<()> {
        self.env_vars = if inherit == Inheritance::None {
            EnvSettings::new()
        } else {
            self.current_env()
        };

        // Add breadcrumbs about which environment.
        self.env_vars.insert(
            CT_ENVIRONMENT.to_string(),
            resolved
                .env_name
                .clone()
                .unwrap_or_else(|| DEFAULT_ENV_NAME.to_string()),
        );

        // Add in the items from the CloudTruth environment (looking for collisions)
        let mut collisions: Vec<String> = vec![];
        let ct_vars = self.get_ct_vars(org_id, &resolved)?;
        let empty = "".to_string();
        for (key, value) in ct_vars {
            if !self.env_vars.contains_key(&key) {
                // when not already, insert it
                self.env_vars.entry(key).or_insert(value);
            } else {
                let orig = self.env_vars.get(&key).unwrap_or(&empty);
                if inherit == Inheritance::Exclusive && value != *orig {
                    collisions.push(key);
                } else if inherit == Inheritance::Overlay {
                    self.env_vars.insert(key, value);
                }
                // if doing Underlay, the local environment value is already set
            }
        }

        // Add in the items from the overrides (looking for collisions)
        let over_vars = self.process_overrides(overrides)?;
        for (key, value) in over_vars {
            let orig = self.env_vars.get(&key).unwrap_or(&empty);
            if inherit == Inheritance::Exclusive
                && self.env_vars.contains_key(&key)
                && value != *orig
            {
                collisions.push(key);
            } else {
                // use the "set" value as the final answer, when not worrying about collisions
                self.env_vars.insert(key, value);
            }
        }

        // return the error(s) if there were not any collisions
        if !collisions.is_empty() {
            Err(SubProcessError::EnvironmentCollisions(collisions))
        } else {
            // Remove the specified values.
            for r in removals {
                self.env_vars.remove(r.as_str());
            }
            Ok(())
        }
    }

    fn remove_ct_app_vars(&mut self) {
        for app_var in CT_APP_REMOVABLE_VARS {
            self.env_vars.remove(*app_var);
        }
    }

    fn run_command(&self, command: &str, arguments: &[String]) -> SubProcessResult<()> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inherit_to_string() {
        let mut map: HashMap<Inheritance, String> = HashMap::new();
        map.insert(Inheritance::None, "none".to_string());
        map.insert(Inheritance::Underlay, "underlay".to_string());
        map.insert(Inheritance::Overlay, "overlay".to_string());
        map.insert(Inheritance::Exclusive, "exclusive".to_string());
        for (iv, sv) in map {
            assert_eq!(format!("{}", iv).to_string(), sv);
        }
    }

    #[test]
    fn inherit_from_string() {
        // Tests case insensitivity, as well as all possible versions
        let mut map: HashMap<String, Result<Inheritance, _>> = HashMap::new();
        map.insert("None".to_string(), Ok(Inheritance::None));
        map.insert("noNe".to_string(), Ok(Inheritance::None));
        map.insert("unDerlay".to_string(), Ok(Inheritance::Underlay));
        map.insert("OVERLAY".to_string(), Ok(Inheritance::Overlay));
        map.insert("exclusive".to_string(), Ok(Inheritance::Exclusive));
        map.insert("Ex-clusive".to_string(), Err(()));
        map.insert("".to_string(), Err(()));
        for (sv, iv) in map {
            assert_eq!(Inheritance::from_str(sv.as_str()), iv);
        }
    }
}
