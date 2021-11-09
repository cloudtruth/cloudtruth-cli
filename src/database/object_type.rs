use cloudtruth_restapi::models::ObjectTypeEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ObjectType {
    AwsIntegration,
    Environment,
    GitHubIntegration,
    Invitation,
    Membership,
    Organization,
    Parameter,
    ParameterRule,
    Project,
    Pull,
    Push,
    ServiceAccount,
    Tag,
    Template,
    Value,
}

const STR_AWS: &str = "aws";
const STR_ENV: &str = "environment";
const STR_GITHUB: &str = "github";
const STR_INVITE: &str = "invitation";
const STR_MEMBER: &str = "membership";
const STR_ORG: &str = "organization";
const STR_PARAM: &str = "parameter";
const STR_RULE: &str = "rule";
const STR_PROJ: &str = "project";
const STR_PULL: &str = "pull";
const STR_PUSH: &str = "push";
const STR_SERV_ACCT: &str = "service-account";
const STR_TAG: &str = "tag";
const STR_TEMP: &str = "template";
const STR_VALUE: &str = "value";

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        let v = match self {
            Self::AwsIntegration => STR_AWS,
            Self::Environment => STR_ENV,
            Self::GitHubIntegration => STR_GITHUB,
            Self::Invitation => STR_INVITE,
            Self::Membership => STR_MEMBER,
            Self::Organization => STR_ORG,
            Self::Parameter => STR_PARAM,
            Self::ParameterRule => STR_RULE,
            Self::Project => STR_PROJ,
            Self::Pull => STR_PULL,
            Self::Push => STR_PUSH,
            Self::ServiceAccount => STR_SERV_ACCT,
            Self::Tag => STR_TAG,
            Self::Template => STR_TEMP,
            Self::Value => STR_VALUE,
        };
        v.to_string()
    }
}

impl From<ObjectTypeEnum> for ObjectType {
    fn from(api: ObjectTypeEnum) -> Self {
        match api {
            ObjectTypeEnum::AwsIntegration => ObjectType::AwsIntegration,
            ObjectTypeEnum::Environment => ObjectType::Environment,
            ObjectTypeEnum::GitHubIntegration => ObjectType::GitHubIntegration,
            ObjectTypeEnum::Invitation => ObjectType::Invitation,
            ObjectTypeEnum::Membership => ObjectType::Membership,
            ObjectTypeEnum::Organization => ObjectType::Organization,
            ObjectTypeEnum::Parameter => ObjectType::Parameter,
            ObjectTypeEnum::ParameterRule => ObjectType::ParameterRule,
            ObjectTypeEnum::Project => ObjectType::Project,
            ObjectTypeEnum::Pull => ObjectType::Pull,
            ObjectTypeEnum::Push => ObjectType::Push,
            ObjectTypeEnum::ServiceAccount => ObjectType::ServiceAccount,
            ObjectTypeEnum::Tag => ObjectType::Tag,
            ObjectTypeEnum::Template => ObjectType::Template,
            ObjectTypeEnum::Value => ObjectType::Value,
        }
    }
}

impl ObjectType {
    pub fn to_server_string(self) -> String {
        let server = match self {
            Self::AwsIntegration => ObjectTypeEnum::AwsIntegration,
            Self::Environment => ObjectTypeEnum::Environment,
            Self::GitHubIntegration => ObjectTypeEnum::GitHubIntegration,
            Self::Invitation => ObjectTypeEnum::Invitation,
            Self::Membership => ObjectTypeEnum::Membership,
            Self::Organization => ObjectTypeEnum::Organization,
            Self::Parameter => ObjectTypeEnum::Parameter,
            Self::ParameterRule => ObjectTypeEnum::ParameterRule,
            Self::Project => ObjectTypeEnum::Project,
            Self::Pull => ObjectTypeEnum::Pull,
            Self::Push => ObjectTypeEnum::Push,
            Self::ServiceAccount => ObjectTypeEnum::ServiceAccount,
            Self::Tag => ObjectTypeEnum::Tag,
            Self::Template => ObjectTypeEnum::Template,
            Self::Value => ObjectTypeEnum::Value,
        };
        server.to_string()
    }
}

pub fn to_object_type(option: Option<&str>) -> Option<ObjectType> {
    match option {
        Some(value) => match value.to_lowercase().as_str() {
            STR_AWS => Some(ObjectType::AwsIntegration),
            STR_ENV => Some(ObjectType::Environment),
            STR_GITHUB => Some(ObjectType::GitHubIntegration),
            STR_INVITE => Some(ObjectType::Invitation),
            STR_MEMBER => Some(ObjectType::Membership),
            STR_ORG => Some(ObjectType::Organization),
            STR_PARAM => Some(ObjectType::Parameter),
            STR_RULE => Some(ObjectType::ParameterRule),
            STR_PROJ => Some(ObjectType::Project),
            STR_PULL => Some(ObjectType::Pull),
            STR_PUSH => Some(ObjectType::Push),
            STR_SERV_ACCT => Some(ObjectType::ServiceAccount),
            STR_TAG => Some(ObjectType::Tag),
            STR_TEMP => Some(ObjectType::Template),
            STR_VALUE => Some(ObjectType::Value),
            _ => None,
        },
        _ => None,
    }
}
