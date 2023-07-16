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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Param {
    pub name: String,
    pub value: String,
    pub raw: String,
}

pub trait ParseParamListExt {
    fn parse_param_list(&self) -> ParamList;
}

impl ParseParamListExt for assert_cmd::assert::Assert {
    fn parse_param_list(&self) -> ParamList {
        serde_json::from_slice::<ParamListRoot>(&self.get_output().stdout)
            .expect("Invalid parameter list JSON")
            .parameter
    }
}
