use cloudtruth_restapi::models::Membership;

#[derive(Clone, Debug)]
pub struct MemberDetails {
    pub id: String,
    pub url: String,
    pub user: String,
    pub role: String,
    pub organization: String,

    pub created_at: String,
    pub modified_at: String,
}

/// Converts from the OpenApi `Member` model to the CloudTruth `MemberDetails`
impl From<&Membership> for MemberDetails {
    fn from(api: &Membership) -> Self {
        Self {
            id: api.id.clone(),
            url: api.url.clone(),
            user: api.user.clone(),
            role: api.role.clone().to_string().to_lowercase(),
            organization: api.organization.clone(),
            created_at: api.created_at.clone(),
            modified_at: api.modified_at.clone(),
        }
    }
}
