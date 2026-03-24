#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use crate::count::Count;
use chrono::{ DateTime, Utc };
use crate::customer::Customer;

pub mod companyCreate;
pub mod companyLocationCreate;
pub mod companyContactCreate;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PurchasingCompanyInput{
    #[serde(rename="companyContactId")]
    pub company_contact_id: Option<String>,             // Shopify GID
    #[serde(rename="companyId")]
    pub company_id: Option<String>,                     // Shopify GID
    #[serde(rename="companyLocationId")]
    pub company_location_id: Option<String>,            // Shopify GID 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompanyContact {
    pub company: Option<Company>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub customer: Option<Customer>,
    pub id: Option<String>,
    #[serde(rename="isMainContact")]
    pub is_main_contact: Option<bool>,
    // ...
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Company {
    // #[serde(rename="contactRoles")]
    // pub contact_roles: Option<CompanyContactRoleConnection>,
    // pub contacts: Option<CompanyContactConnection>
    pub id: Option<String>,
    #[serde(rename="contactsCount")]
    pub contacts_count: Option<Count>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename="customerSince")]
    pub customer_since: Option<DateTime<Utc>>,
    #[serde(rename="defaultCursor")]
    pub default_cursor: Option<String>,
    pub name: Option<String>,
    // ...
}