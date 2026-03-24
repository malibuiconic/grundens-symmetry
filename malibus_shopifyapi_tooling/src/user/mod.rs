use serde::{ Deserialize, Serialize };
use crate::image::Image;
use chrono::{ DateTime, Utc };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaffMember {
    #[serde(rename="accountType")]
    pub account_type: Option<AccountType>,
    pub active: Option<bool>,
    pub avatar: Option<Image>,
    pub email: Option<String>,
    pub exists: Option<bool>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    pub id: Option<String>,
    pub initials: Option<Vec<String>>,
    #[serde(rename="isShopOwner")]
    pub is_shop_owner: Option<bool>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub locale: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    #[serde(rename="privateData")]
    pub private_data: Option<StaffMemberPrivateData>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaffMemberPrivateData {
    #[serde(rename="accountSettingsUrl")]
    pub account_settings_url: Option<String>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AccountType {
    Collaborator,
    CollaboratorTeamMember,
    Invited,
    InvitedStoreOwner,
    Regular,
    Requested,
    Restricted,
    Saml,
}

impl AccountType {
    pub fn from_str(status: &str) -> AccountType {
        match status {
            "COLLABORATOR" => AccountType::Collaborator,
            "COLLABORATOR_TEAM_MEMBER" => AccountType::CollaboratorTeamMember,
            "INVITED" => AccountType::Invited,
            "INVITED_STORE_OWNER" => AccountType::InvitedStoreOwner,
            "REGULAR" => AccountType::Regular,
            "REQUESTED" => AccountType::Requested,
            "RESTRICTED" => AccountType::Restricted,
            "SAML" | _ => AccountType::Saml,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AccountType::Collaborator => String::from("COLLABORATOR"),
            AccountType::CollaboratorTeamMember => String::from("COLLABORATOR_TEAM_MEMBER"),
            AccountType::Invited => String::from("INVITED"),
            AccountType::InvitedStoreOwner => String::from("INVITED_STORE_OWNER"),
            AccountType::Regular => String::from("REGULAR"),
            AccountType::Requested => String::from("REQUESTED"),
            AccountType::Restricted => String::from("RESTRICTED"),
            AccountType::Saml => String::from("SAML"),
        }
    }
}

