use std::ops::{Deref, DerefMut};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ParamTypesListRoot<T> {
    #[serde(rename = "parameter-type")]
    parameter_type: ParamTypesList<T>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamTypesList<T>(Vec<T>);

impl<T> Deref for ParamTypesList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ParamTypesList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParamTypeEntry {
    pub description: String,
    pub name: String,
    pub parent: String,
    pub rules: String,
    #[serde(rename = "Created At")]
    pub created_at: Option<String>,
    #[serde(rename = "Modified At")]
    pub modified_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConstraintEntry {
    pub constraint: String,
    pub name: String,
    pub parent: String,
    #[serde(rename = "Rule Type")]
    pub rule_type: String,
    #[serde(rename = "Created At")]
    pub created_at: Option<String>,
    #[serde(rename = "Modified At")]
    pub modified_at: Option<String>,
}

pub trait ParseParamTypesExt {
    fn parse_param_types_list(&self) -> ParamTypesList<ParamTypeEntry>;
    fn parse_param_types_list_with_rules(&self) -> ParamTypesList<ConstraintEntry>;
}

impl ParseParamTypesExt for assert_cmd::assert::Assert {
    fn parse_param_types_list(&self) -> ParamTypesList<ParamTypeEntry> {
        serde_json::from_slice::<ParamTypesListRoot<ParamTypeEntry>>(&self.get_output().stdout)
            .expect("Unable to parse JSON as param types list")
            .parameter_type
    }
    fn parse_param_types_list_with_rules(&self) -> ParamTypesList<ConstraintEntry> {
        serde_json::from_slice::<ParamTypesListRoot<ConstraintEntry>>(&self.get_output().stdout)
            .expect("Unable to parse JSON as param types list")
            .parameter_type
    }
}
