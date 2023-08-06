use std::ops::{Deref, DerefMut};

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
