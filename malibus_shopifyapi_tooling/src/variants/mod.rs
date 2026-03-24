#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use chrono::{ DateTime, Utc };
use crate::money::Money;
use crate::image::Image;
use crate::metafields::MetafieldInput;
use crate::selling_plans::SellingPlanGroupConnection;
use crate::metafields::MetafieldConnection;

pub mod productVariantsBulkCreate;
pub mod productVariantsBulkUpdate;
pub mod productOptionsCreate;
pub mod getProductVariantMetafields;

// https://shopify.dev/docs/api/admin-graphql/2025-01/mutations/productVariantsBulkCreate
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductVariantsBulkInput {
    // pub barcode: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(rename="compareAtPrice")]
    // pub compare_at_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    // ... add other fields when needed
    pub price: Option<f64>,
    #[serde(rename="optionValues")]
    pub option_values: Option<Vec<VariantOptionValueInput>>,
    #[serde(rename="inventoryItem")]
    pub inventory_item: Option<InventoryItem>,
    // ... add other fields when needed
    pub metafields: Option<Vec<MetafieldInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InventoryItem {
    // pub sku: Option<String>                    // Will likely associate with the NetsuiteID
    #[serde(rename="requiresShipping")]
    pub requires_shipping: Option<bool>,
    pub tracked: Option<bool>,     
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariantOptionValueInput {
    pub name: Option<String>,
    #[serde(rename="optionName")]
    pub option_name: Option<String>,            // Required or optionID (NOTE you need to have options first created btw!)
     #[serde(rename="optionId")]
     #[serde(skip_serializing_if = "Option::is_none")]
    pub option_id: Option<String>,              // Gid.
    // ...
}


// Standard ProductVariant
// https://shopify.dev/docs/api/admin-graphql/2025-01/objects/ProductVariant
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariant {
    pub barcode: Option<String>,
    #[serde(rename="compareAtPrice")]
    pub compare_at_price: Option<Money>,
    #[serde(rename="createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename="defaultCursor")]
    pub default_cursor: Option<String>,
    // #[serde(rename="deliveryProfile")]                 // Shipping
    // pub delivery_profile: Option<DeliveryProfile>,
    #[serde(rename="displayName")]
    pub display_name: Option<String>,
    // pub events: Option<EventConnection>,
    pub id: Option<String>,
    pub image: Option<Image>,
    // #[serde(rename="inventoryItem")]
    // pub inventory_item: Option<InventoryItem>,
    // #[serde(rename="inventoryPolicy")]
    // pub inventory_policy: Option<ProductVariantInventoryPolicy>,
    #[serde(rename="inventoryQuantity")]
    pub inventory_quantity: Option<u64>,
    // pub media: Option<MediaConnection>,
    // pub metafield: Option<Metafield>,
    pub metafields: Option<MetafieldConnection>,
    // pub position: Option<u64>,
    pub price: Option<Money>,
    // pub product: Option<Product>,
    // #[serde(rename="productVariantComponents")]
    // pub product_variant_component: Option<ProductVariantComponentConnection>
    // #[serde(rename="requiresComponents")]
    // pub requires_components: Option<bool>,
    // #[serde(rename="selectedOptions")]
    // pub selected_options: Option<Vec<SelectedOption>>,
    // #[serde(rename="sellableOnlineQuantity")]
    // pub sellable_online_quantity: Option<u64>,
    #[serde(rename="sellingPlanGroups")]
    pub selling_plan_groups: Option<Box<SellingPlanGroupConnection>>,  // Box here creates a pointer to the data instead of embedding it directly in struct
    // #[serde(rename="sellingPlanGroupsCount")]
    // pub selling_plan_groups_count: Option<Count>,
    pub sku: Option<String>,
    pub taxable: Option<bool>,
    #[serde(rename="taxCode")]
    pub tax_code: Option<String>,
    pub title: Option<String>,
    // #[serde(rename="translations")]
    // pub translations: Option<Vec<Translation>>,
    // #[serde(rename="unitPriceMeasurement")]
    // pub unit_price_measurement: Option<UnitPriceMeasurement>,
    #[serde(rename="updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}
