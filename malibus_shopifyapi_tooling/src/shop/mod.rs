#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use chrono::prelude::*;

pub mod getShopResourceLimits;

// https://shopify.dev/docs/api/admin-graphql/2024-07/queries/shop
// https://shopify.dev/docs/api/admin-graphql/2024-07/objects/Shop
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shop {
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub id: Option<u64>,
    pub plan: Option<ShopPlan>,
    #[serde(rename="resourceLimits")]
    pub resource_limits: Option<ShopResourceLimits>,
    // .. Add others as needed!
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopPlan {
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    #[serde(rename="partnerDevelopment")]
    pub partner_development: Option<bool>,
    #[serde(rename="shopifyPlus")]
    pub shopify_plus: Option<bool>
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopResourceLimits {
    #[serde(rename="locationLimit")]
    pub location_limit: Option<u32>,
    #[serde(rename="maxProductOptions")]
    pub max_product_options: Option<u32>,
    #[serde(rename="maxProductVariants")]
    pub max_product_variants: Option<u32>,
    #[serde(rename="redirectLimitReached")]
    pub redirect_limit_reached: Option<u32>,
}