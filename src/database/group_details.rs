use cloudtruth_restapi::models::Group;

use super::UserNameMap;

#[derive(Clone, Debug)]
pub struct GroupDetails {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
    pub users: Vec<String>,

    pub created_at: String,
    pub modified_at: String,
}

impl GroupDetails {
    pub fn new(group: &Group, user_map: &UserNameMap) -> Self {
        Self {
            id: group.id.clone(),
            url: group.url.clone(),
            name: group.name.clone(),
            description: group.description.clone().unwrap_or_default(),
            users: group
                .users
                .iter()
                .filter_map(|user_url| user_map.get(user_url).map(String::from))
                .collect::<Vec<String>>(),
            created_at: group.created_at.clone(),
            modified_at: group.modified_at.clone(),
        }
    }
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "users" => self.users.join(", "),
            "created-at" => self.created_at.clone(),
            "modified-at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{property_name}'"),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}
