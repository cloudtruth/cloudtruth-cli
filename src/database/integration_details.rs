use cloudtruth_restapi::models::{AwsIntegration, AzureKeyVaultIntegration, GitHubIntegration};

#[derive(Clone, Debug)]
pub struct IntegrationDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub fqn: String,
    pub status: String,
    pub status_detail: String,
    pub status_time: String,
    pub created_at: String,
    pub modified_at: String,
}

impl IntegrationDetails {
    pub fn get_property(&self, property_name: &str) -> String {
        match property_name {
            "id" => self.id.clone(),
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "provider" => self.provider.clone(),
            "fqn" => self.fqn.clone(),
            "status" => self.status.clone(),
            "status-detail" => self.status_detail.clone(),
            "status-time" => self.status_time.clone(),
            "created-at" => self.created_at.clone(),
            "modified_at" => self.modified_at.clone(),
            _ => format!("Unhandled property name '{property_name}'"),
        }
    }

    pub fn get_properties(&self, fields: &[&str]) -> Vec<String> {
        fields.iter().map(|p| self.get_property(p)).collect()
    }
}

impl From<&AwsIntegration> for IntegrationDetails {
    fn from(aws: &AwsIntegration) -> Self {
        IntegrationDetails {
            id: aws.id.clone(),
            provider: "aws".to_string(),
            name: aws.name.clone(),
            description: aws.description.clone().unwrap_or_default(),
            fqn: aws.fqn.clone(),
            status: match &aws.status {
                Some(s) => s.to_string(),
                _ => "".to_string(),
            },
            status_detail: aws.status_detail.clone(),
            status_time: aws.status_last_checked_at.clone(),
            created_at: aws.created_at.clone(),
            modified_at: aws.modified_at.clone(),
        }
    }
}

impl From<&GitHubIntegration> for IntegrationDetails {
    fn from(github: &GitHubIntegration) -> Self {
        IntegrationDetails {
            id: github.id.clone(),
            provider: "github".to_string(),
            name: github.name.clone(),
            description: github.description.clone().unwrap_or_default(),
            fqn: github.fqn.clone(),
            status: match &github.status {
                Some(s) => s.to_string(),
                _ => "".to_string(),
            },
            status_detail: github.status_detail.clone(),
            status_time: github.status_last_checked_at.clone(),
            created_at: github.created_at.clone(),
            modified_at: github.modified_at.clone(),
        }
    }
}

impl From<&AzureKeyVaultIntegration> for IntegrationDetails {
    fn from(azure: &AzureKeyVaultIntegration) -> Self {
        IntegrationDetails {
            id: azure.id.clone(),
            name: azure.name.clone(),
            description: azure.description.clone().unwrap_or_default(),
            provider: "azure".to_string(),
            fqn: azure.fqn.clone(),
            status: match &azure.status {
                Some(s) => s.to_string(),
                _ => "".to_string(),
            },
            status_detail: azure.status_detail.clone(),
            status_time: azure.status_last_checked_at.clone(),
            created_at: azure.created_at.clone(),
            modified_at: azure.modified_at.clone(),
        }
    }
}
