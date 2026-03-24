use serde::{Deserialize, Serialize};
use chrono::{ DateTime, Utc };
use crate::address::MailingAddress;
use crate::money::MoneyV2;
use crate::company::CompanyContact;
use crate::orders::Order;
use crate::PageInfo;
use crate::metafields::MetafieldConnection;
use crate::payments::CustomerPaymentMethodConnection;

pub mod storefrontAPI;   
pub mod graphQL;

// https://shopify.dev/docs/api/admin-graphql/2025-04/objects/Customer
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Customer {
    pub addresses: Option<Vec<MailingAddress>>,
    #[serde(rename="addressesV2")]
    pub addresses_v2: Option<MailingAddressConnection>,
    #[serde(rename="amountSpent")]
    pub amount_spent: Option<MoneyV2>,
    #[serde(rename="canDelete")]
    pub can_delete: Option<bool>,
    #[serde(rename="companyContactProfiles")]
    pub company_contact_profiles: Option<Vec<CompanyContact>>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename="dataSaleOptOut")]
    pub data_sale_opt_out: Option<bool>,
    #[serde(rename="defaultAddress")]
    pub default_address: Option<MailingAddress>,
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub metafields: Option<MetafieldConnection>,
    // #[serde(rename="lastOrder")]
    // pub last_order: Option<Order>,
     // .. Other Fields
    pub id: Option<String>,
    pub note: Option<String>,
    // .. Other Fields
    #[serde(rename="paymentMethods")]
    pub payment_methods: Option<CustomerPaymentMethodConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MailingAddressConnection {
    pub edges: Option<Vec<MailingAddressEdge>>,
    // pub nodes: Option<Vec<MailingAddress>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MailingAddressEdge {
    pub node: Option<MailingAddress>,
    pub cursor: Option<String>,
}
