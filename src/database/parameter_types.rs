use cloudtruth_restapi::models::ParameterTypeEnum;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    String,
    Bool,
    Integer,
}

impl From<ParameterTypeEnum> for ParamType {
    fn from(api: ParameterTypeEnum) -> Self {
        match api {
            ParameterTypeEnum::Bool => Self::Bool,
            ParameterTypeEnum::String => Self::String,
            ParameterTypeEnum::Integer => Self::Integer,
        }
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Integer => write!(f, "integer"),
            Self::String => write!(f, "string"),
        }
    }
}

impl ParamType {
    pub fn to_api_enum(&self) -> ParameterTypeEnum {
        match self {
            Self::String => ParameterTypeEnum::String,
            Self::Bool => ParameterTypeEnum::Bool,
            Self::Integer => ParameterTypeEnum::Integer,
        }
    }
}
