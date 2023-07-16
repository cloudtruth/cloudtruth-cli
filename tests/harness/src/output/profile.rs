use std::ops::{Deref, DerefMut};

use serde::Deserialize;

use crate::cloudtruth;

#[derive(Debug, Clone, Deserialize)]
struct ProfileRoot<T> {
    profile: T,
}

type CurrentProfileRoot = ProfileRoot<CurrentProfileParams>;

#[derive(Debug, Clone, Deserialize)]
pub struct CurrentProfileParams(Vec<CurrentProfileParam>);

impl std::ops::Deref for CurrentProfileParams {
    type Target = Vec<CurrentProfileParam>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CurrentProfileParams {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CurrentProfileParams {
    pub fn find_param(&self, name: &str) -> &CurrentProfileParam {
        self.0
            .iter()
            .find(|p| p.parameter == name)
            .unwrap_or_else(|| panic!("Could not find parameter '{name}' in profile JSON"))
    }

    pub fn param_names(&self) -> Vec<&str> {
        self.0
            .iter()
            .map(|param| param.parameter.as_str())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrentProfileParam {
    pub parameter: String,
    pub source: String,
    pub value: String,
}

/// Use an extension trait to extend the assert_cmd::Assert type with a method for parsing the config JSON
pub trait ParseCurrentProfileParamsExt {
    fn parse_current_profile_params(&self) -> CurrentProfileParams;
}

impl ParseCurrentProfileParamsExt for assert_cmd::assert::Assert {
    fn parse_current_profile_params(&self) -> CurrentProfileParams {
        serde_json::from_slice::<CurrentProfileRoot>(&self.get_output().stdout)
            .expect("Invalid profile JSON")
            .profile
    }
}
type ProfileListRoot = ProfileRoot<ProfileList>;

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileList(Vec<ProfileListEntry>);

impl Deref for ProfileList {
    type Target = Vec<ProfileListEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProfileList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ProfileList {
    pub fn find_by_name(&self, name: &str) -> Option<&ProfileListEntry> {
        self.iter().find(|prof| prof.name == name)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProfileListEntry {
    #[serde(rename = "API")]
    pub api: String,
    pub description: String,
    pub environment: String,
    pub name: String,
    pub project: String,
}

pub trait ParseProfileListExt {
    fn parse_profile_list(&self) -> ProfileList;
}

impl ParseProfileListExt for assert_cmd::assert::Assert {
    fn parse_profile_list(&self) -> ProfileList {
        serde_json::from_slice::<ProfileListRoot>(&self.get_output().stdout)
            .expect("Invalid profile list JSON")
            .profile
    }
}

pub fn get_current_user() -> String {
    let profile = cloudtruth!("config current -f json")
        .assert()
        .success()
        .parse_current_profile_params();
    profile.find_param("User").value.to_string()
}
