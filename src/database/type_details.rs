use crate::database::{ParamRuleType, ParameterRuleDetail};
use cloudtruth_restapi::models::ParameterType;

#[derive(Clone, Debug)]
pub struct TypeDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub parent_name: String,
    pub parent_url: String,
    pub rules: Vec<ParameterRuleDetail>,
    pub created_at: String,
    pub modified_at: String,
}

/// Converts from the OpenApi `ParameterType` model to the CloudTruth `TypeDetails`
impl From<&ParameterType> for TypeDetails {
    fn from(api_ptype: &ParameterType) -> Self {
        Self {
            id: api_ptype.id.clone(),
            url: api_ptype.url.clone(),
            name: api_ptype.name.clone(),
            description: api_ptype.description.clone().unwrap_or_default(),
            parent_url: api_ptype.parent.clone().unwrap_or_default(),
            parent_name: api_ptype.parent_name.clone().unwrap_or_default(),
            rules: vec![],
            created_at: api_ptype.created_at.clone(),
            modified_at: api_ptype.modified_at.clone(),
        }
    }
}

impl TypeDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "parent-url" => self.parent_url.clone(),
            "parent-name" => self.parent_name.clone(),
            "rule-count" => format!("{}", self.rules.len()),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{}'", property_name),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }

    /// Gets the first id matching the provided type
    pub fn get_rule_id(&self, rule_type: ParamRuleType) -> Option<String> {
        let mut result: Option<String> = None;
        for rule in &self.rules {
            if rule.rule_type == rule_type {
                result = Some(rule.id.clone());
                break;
            }
        }
        result
    }
}
