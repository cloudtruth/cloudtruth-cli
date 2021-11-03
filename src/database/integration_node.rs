use crate::database::last_from_url;
use cloudtruth_restapi::models::IntegrationExplorer;

#[derive(Debug)]
pub struct IntegrationNode {
    pub fqn: String,
    pub node_type: String,
    pub secret: bool,
    pub name: String,
    pub content_type: String,
    pub content_size: i32,
    pub content_data: String,
    pub content_keys: Vec<String>,
}

fn get_name(name: &Option<String>, fqn: &str) -> String {
    if let Some(name) = name {
        name.clone()
    } else {
        last_from_url(fqn).to_string()
    }
}

impl From<&IntegrationExplorer> for IntegrationNode {
    fn from(node: &IntegrationExplorer) -> Self {
        IntegrationNode {
            fqn: node.fqn.clone(),
            name: get_name(&node.name, &node.fqn),
            node_type: format!("{:?}", node.node_type),
            secret: node.secret.unwrap_or(false),
            content_type: node.content_type.clone().unwrap_or_default(),
            content_size: node.content_size.unwrap_or(0),
            content_data: node.content_data.clone().unwrap_or_default(),
            content_keys: node.content_keys.clone().unwrap_or_default(),
        }
    }
}
