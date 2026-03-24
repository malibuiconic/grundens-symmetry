#![allow(non_snake_case)]
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use std::fmt::Debug;
use std::fmt;
use serde::de::{self, Visitor};
use serde_json::json;
use crate::error::UserError;

pub mod customerAccessToken;   // is a required action if wanting to update customer data 
pub mod customerCreate;
pub mod customerAddress;       // Address Functions
pub mod customerUpdate;

// https://shopify.dev/docs/api/storefront/2024-07/mutations/customerCreate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerCreateInput {
    #[serde(rename="acceptsMarketing")]
    pub accepts_marketing: Option<bool>,
    pub email: Option<String>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub password: Option<String>,      // the login password used by the customer
    pub phone: Option<String>,
}