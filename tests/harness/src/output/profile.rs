use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileRoot {
    profile: ProfileParameters,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileParameters(Vec<ProfileParameter>);

impl std::ops::Deref for ProfileParameters {
    type Target = Vec<ProfileParameter>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ProfileParameters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ProfileParameters {
    pub fn find_param(&self, name: &str) -> &ProfileParameter {
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
pub struct ProfileParameter {
    pub parameter: String,
    pub source: String,
    pub value: String,
}

/// Use an extension trait to extend the assert_cmd::Assert type with a method for parsing the config JSON
pub trait ParseProfileParametersExt {
    fn parse_profile_parameters(&self) -> ProfileParameters;
}

impl ParseProfileParametersExt for assert_cmd::assert::Assert {
    fn parse_profile_parameters(&self) -> ProfileParameters {
        serde_json::from_slice::<ProfileRoot>(&self.get_output().stdout)
            .expect("Invalid profile JSON")
            .profile
    }
}
