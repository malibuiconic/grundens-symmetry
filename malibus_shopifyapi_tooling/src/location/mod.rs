#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use chrono::{DateTime, Utc};
use crate::metafields::Metafield;
use crate::metafields::MetafieldConnection;

// https://shopify.dev/docs/api/admin-graphql/latest/objects/Location
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub activatable: Option<bool>,
    pub address: Option<LocationAddress>,
    #[serde(rename="addressVerified")]
    pub address_verified: Option<bool>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub deactivatable: Option<bool>,
    #[serde(rename="deactivatedAt")]
    pub deactivated_at: Option<DateTime<Utc>>,
    pub deletable: Option<bool>,
    // #[serde(rename="fulfillmentService")]
    // pub fulfillment_service: Option<FulfillmentmentService>,
    #[serde(rename = "fulfillsOnlineOrders")]
    pub fulfills_online_orders: Option<bool>,
    #[serde(rename = "hasActiveInventory")]
    pub has_active_inventory: Option<bool>,
    #[serde(rename = "hasUnfulfilledOrders")]
    pub has_unfulfilled_orders: Option<bool>,
    id: Option<String>,
    // #[serde(rename = "inventoryLevel")]
    // pub inventory_level: Option<InventoryLevel>,
    // #[serde(rename = "inventoryLevels")]
    // pub inventory_levels: Option<InventoryLevelConnection>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
    #[serde(rename = "isFulfillmentService")]
    pub is_fulfillment_service: Option<bool>,
    #[serde(rename = "legacyResourceId")]
    pub legacy_resource_id: Option<u64>,
    // #[serde(rename = "localPickupSettingsV2")]
    // pub local_pickup_settings: Option<DeliveryLocalPickupSettings>,
    pub metafield: Option<Metafield>,
    pub metafields: Option<MetafieldConnection>,
    pub name: Option<String>,
    #[serde(rename = "shipsInventory")]
    pub ships_inventory: Option<bool>,
    // #[serde(rename = "suggestedAddresses")]
    // pub suggested_addresses: Option<Vec<LocationSuggestedAddress>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocationAddress {
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
    pub formatted: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub phone: Option<String>,
    pub province: Option<String>,
    #[serde(rename = "provinceCode")]
    pub province_code: Option<String>,
    pub zip: Option<String>,
}