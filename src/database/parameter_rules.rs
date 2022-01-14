use cloudtruth_restapi::models::{ParameterRule, ParameterRuleTypeEnum, ParameterTypeRule};
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParamRuleType {
    Max,
    Min,
    MaxLen,
    MinLen,
    Regex,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ParameterRuleDetail {
    pub id: String,
    pub rule_type: ParamRuleType,
    pub constraint: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<ParameterRuleTypeEnum> for ParamRuleType {
    fn from(api: ParameterRuleTypeEnum) -> Self {
        match api {
            ParameterRuleTypeEnum::Max => Self::Max,
            ParameterRuleTypeEnum::Min => Self::Min,
            ParameterRuleTypeEnum::MaxLen => Self::MaxLen,
            ParameterRuleTypeEnum::MinLen => Self::MinLen,
            ParameterRuleTypeEnum::Regex => Self::Regex,
            ParameterRuleTypeEnum::UnknownDefaultOpenApi => Self::Unknown,
        }
    }
}

impl From<ParamRuleType> for ParameterRuleTypeEnum {
    fn from(ct: ParamRuleType) -> Self {
        match ct {
            ParamRuleType::Max => Self::Max,
            ParamRuleType::Min => Self::Min,
            ParamRuleType::MaxLen => Self::MaxLen,
            ParamRuleType::MinLen => Self::MinLen,
            ParamRuleType::Regex => Self::Regex,
            ParamRuleType::Unknown => Self::UnknownDefaultOpenApi,
        }
    }
}

impl fmt::Display for ParamRuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Max => write!(f, "max"),
            Self::Min => write!(f, "min"),
            Self::MaxLen => write!(f, "max-len"),
            Self::MinLen => write!(f, "min-len"),
            Self::Regex => write!(f, "regex"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&ParameterRule> for ParameterRuleDetail {
    fn from(api: &ParameterRule) -> Self {
        Self {
            id: api.id.clone(),
            rule_type: ParamRuleType::from(api._type),
            constraint: api.constraint.clone(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}

impl From<&ParameterTypeRule> for ParameterRuleDetail {
    fn from(api: &ParameterTypeRule) -> Self {
        Self {
            id: api.id.clone(),
            rule_type: ParamRuleType::from(api._type),
            constraint: api.constraint.clone(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
