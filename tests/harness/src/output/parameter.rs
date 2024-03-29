use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct ParamListRoot {
    parameter: ParamList,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamList(Vec<Param>);

impl Deref for ParamList {
    type Target = Vec<Param>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParamList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ParamList {
    pub fn find_by_name(&self, name: &str) -> Option<&Param> {
        self.iter().find(|param| param.name == name)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Param {
    pub name: String,
    pub value: String,
    pub raw: Option<String>,
    pub secret: Option<String>,
    #[serde(rename = "Modified At")]
    pub modified_at: Option<String>,
    #[serde(rename = "Created At")]
    pub created_at: Option<String>,
}

pub trait ParseParamListExt {
    fn parse_param_list(&self) -> ParamList;
    fn get_param(&self, name: &str) -> Option<Param> {
        let mut params = self.parse_param_list();
        let (index, _) = params
            .iter()
            .enumerate()
            .find(|(_, param)| param.name == name)?;
        Some(params.swap_remove(index))
    }
}

impl ParseParamListExt for assert_cmd::assert::Assert {
    fn parse_param_list(&self) -> ParamList {
        serde_json::from_slice::<ParamListRoot>(&self.get_output().stdout)
            .expect("Invalid parameter list JSON")
            .parameter
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ParamDiffRoot {
    parameter: ParamDiff,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamDiff(Vec<ParamDiffEntry>);

impl Deref for ParamDiff {
    type Target = Vec<ParamDiffEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParamDiff {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamDiffEntry {
    #[serde(rename = "Parameter")]
    pub parameter: String,
    #[serde(flatten)]
    pub envs: IndexMap<String, String>,
}

impl ParamDiffEntry {
    pub fn split_fields(&self, env: impl AsRef<str>) -> impl Iterator<Item = Cow<str>> {
        self.envs[env.as_ref()]
            .split(',')
            .map(|s| s.replace(|c| c == '"' || c == '\n', "").into())
    }
}

pub trait ParseParamDiffExt {
    fn parse_param_diff(&self) -> ParamDiff;
}

impl ParseParamDiffExt for assert_cmd::assert::Assert {
    fn parse_param_diff(&self) -> ParamDiff {
        serde_json::from_slice::<ParamDiffRoot>(&self.get_output().stdout)
            .expect("Invalid parameter diff JSON")
            .parameter
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ParamDriftRoot {
    #[serde(rename = "parameter-drift")]
    parameter_drift: ParamDrift,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamDrift(Vec<ParamDriftEntry>);

impl Deref for ParamDrift {
    type Target = Vec<ParamDriftEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParamDrift {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ParamDrift {
    pub fn find_by_name(&self, name: &str) -> Option<&ParamDriftEntry> {
        self.iter().find(|param| param.name == name)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParamDriftEntry {
    pub name: String,
    #[serde(rename = "CloudTruth")]
    pub cloudtruth: String,
    pub shell: String,
    pub difference: String,
}

pub trait ParseParamDriftExt {
    fn parse_param_drift(&self) -> ParamDrift;
}

impl ParseParamDriftExt for assert_cmd::assert::Assert {
    fn parse_param_drift(&self) -> ParamDrift {
        serde_json::from_slice::<ParamDriftRoot>(&self.get_output().stdout)
            .expect("Invalid parameter drift JSON")
            .parameter_drift
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ParamEnvRoot {
    parameter: ParamEnv,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamEnv(Vec<ParamEnvEntry>);

impl Deref for ParamEnv {
    type Target = Vec<ParamEnvEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParamEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ParamEnv {
    pub fn find_by_env(&self, env_name: impl AsRef<str>) -> Option<&ParamEnvEntry> {
        self.iter()
            .find(|param| param.environment == env_name.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParamEnvEntry {
    pub value: String,
    pub raw: Option<String>,
    #[serde(rename = "Modified At")]
    pub modified_at: Option<String>,
    #[serde(rename = "Created At")]
    pub created_at: Option<String>,
    pub environment: String,
}

pub trait ParseParamEnvExt {
    fn parse_param_env(&self) -> ParamEnv;
}

impl ParseParamEnvExt for assert_cmd::assert::Assert {
    fn parse_param_env(&self) -> ParamEnv {
        serde_json::from_slice::<ParamEnvRoot>(&self.get_output().stdout)
            .expect("Invalid parameter drift JSON")
            .parameter
    }
}
