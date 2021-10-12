use cloudtruth_restapi::models::ObjectTypeEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ObjectType {
    DataIntegration,
    Environment,
    Parameter,
    ParameterRule,
    Project,
    Template,
    Value,
}

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            Self::DataIntegration => String::from("integration"),
            Self::Environment => String::from("environment"),
            Self::Parameter => String::from("parameter"),
            Self::ParameterRule => String::from("rule"),
            Self::Project => String::from("project"),
            Self::Template => String::from("template"),
            Self::Value => String::from("value"),
        }
    }
}

impl From<ObjectTypeEnum> for ObjectType {
    fn from(api: ObjectTypeEnum) -> Self {
        match api {
            ObjectTypeEnum::DataIntegration => ObjectType::DataIntegration,
            ObjectTypeEnum::Environment => ObjectType::Environment,
            ObjectTypeEnum::Parameter => ObjectType::Parameter,
            ObjectTypeEnum::ParameterRule => ObjectType::ParameterRule,
            ObjectTypeEnum::Project => ObjectType::Project,
            ObjectTypeEnum::Template => ObjectType::Template,
            ObjectTypeEnum::Value => ObjectType::Value,
        }
    }
}

impl ObjectType {
    pub fn to_server_string(self) -> String {
        let server = match self {
            Self::DataIntegration => ObjectTypeEnum::DataIntegration,
            Self::Environment => ObjectTypeEnum::Environment,
            Self::Parameter => ObjectTypeEnum::Parameter,
            Self::ParameterRule => ObjectTypeEnum::ParameterRule,
            Self::Project => ObjectTypeEnum::Project,
            Self::Template => ObjectTypeEnum::Template,
            Self::Value => ObjectTypeEnum::Value,
        };
        server.to_string()
    }
}

pub fn to_object_type(option: Option<&str>) -> Option<ObjectType> {
    match option {
        Some(value) => match value.to_lowercase().as_str() {
            "environment" => Some(ObjectType::Environment),
            "integration" => Some(ObjectType::DataIntegration),
            "rule" => Some(ObjectType::ParameterRule),
            "parameter" => Some(ObjectType::Parameter),
            "project" => Some(ObjectType::Project),
            "template" => Some(ObjectType::Template),
            "value" => Some(ObjectType::Value),
            _ => None,
        },
        _ => None,
    }
}
