#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use crate::metafields::MetafieldInput;
use crate::metafields::MetafieldConnection;

pub mod productCreate;                     // Main Product (basic create)
pub mod productSet;                        // Idempotent upsert (ERP sync — preferred)
pub mod getProductMetafields;              // Extract the Products' Metafields
pub mod getProductTags;                    // Extract the Products' Tags

// PRODUCT
// https://shopify.dev/docs/api/admin-graphql/2023-10/mutations/productCreate
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProductInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="claimOwnership")]
    pub claim_ownership: Option<ProductClaimOwnershipInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<TaxonomyCategory>,
    #[serde(rename="collectionsToJoin")]
    pub collections_to_join: Option<String>,    //[ID!] should be array of strings
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="collectionsToLeave")]  
    pub collections_to_leave: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="combinedListingRole")]
    pub combined_listing_role: Option<CombinedListingRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="customProductType")]
    pub custom_product_type: Option<String>,
    #[serde(rename="descriptionHtml")]
    pub description_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="giftCard")]
    pub gift_card: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="giftCardTemplateSuffix")]
    pub gift_card_template_suffix: Option<String>,
    pub handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="id")]
    pub gid: Option<String>,
    pub metafields: Option<Vec<MetafieldInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="productOptions")]
    pub product_options: Option<Vec<OptionCreateInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="productType")]
    pub product_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="redirectNewHandle")]
    pub redirect_new_handle: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="requiresSellingPlan")]
    pub requires_selling_plan: Option<bool>,
    pub seo: Option<SEOInput>,
    pub status: Option<ProductStatus>,
    pub tags: Option<String>,                   // A Vec of Strings to String
    #[serde(rename="templateSuffix")]
    pub template_suffix: Option<String>,
    pub title: Option<String>,
    pub vendor: Option<String>,
}

// https://shopify.dev/docs/api/admin-graphql/2024-04/objects/TaxonomyCategory#fields
// https://shopify.github.io/product-taxonomy/releases/unstable/?categoryId=gid%3A%2F%2Fshopify%2FTaxonomyCategory%2Faa
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxonomyCategory {
    #[serde(rename="productTaxonomyNodeId")]
    pub id: Option<String>,    // non-null required (gid://Shopify/TaxonomyCategory/<globally unique ID>)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="fullName")]
    pub full_name: Option<String>,
    // .. there are other fields
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CombinedListingRole {
    CHILD,
    PARENT,
}

impl CombinedListingRole {
    pub fn from_str(role: &str) -> CombinedListingRole {
        match role {
            "CHILD" => CombinedListingRole::CHILD,
            "PARENT" | _=> CombinedListingRole::PARENT,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            CombinedListingRole::CHILD => String::from("CHILD"),
            CombinedListingRole::PARENT => String::from("PARENT"),
        }
    }
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProductStatus{
    ACTIVE,
    ARCHIVED,
    DRAFT,
}

impl ProductStatus {
    pub fn from_str(role: &str) -> ProductStatus {
        match role {
            "ACTIVE" => ProductStatus::ACTIVE,              
            "ARCHIVED" => ProductStatus::ARCHIVED,
            "DRAFT" | _=> ProductStatus::DRAFT,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            ProductStatus::ACTIVE => String::from("ACTIVE"),
            ProductStatus::ARCHIVED => String::from("ARCHIVED"),
            ProductStatus::DRAFT => String::from("DRAFT"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SEOInput {
    pub description: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OptionCreateInput {
   #[serde(skip_serializing_if = "Option::is_none")]
   #[serde(rename="linkedMetafield")]
   pub linked_metafield: Option<LinkedMetafieldCreateInput>,
   pub name: Option<String>,
   pub position: Option<u32>,
   pub values: Option<Vec<OptionValueCreateInput>>
}

#[derive(Debug, Clone, Deserialize, Serialize, Default )]
pub struct OptionValueCreateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="linkedMetafieldValue")]
    pub linked_metafield_value: Option<String>,
    pub name: Option<String>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkedMetafieldCreateInput {
    pub key: String,             // required
    pub namespace: String,       // required
    pub values: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductClaimOwnershipInput {
    pub bundles: Option<bool>,
}

// PRODUCT VARIANT
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantInput {
    pub barcode: Option<String>,
    #[serde(rename="compareAtPrice")]
    pub compare_at_price: Option<f32>,          // Money
    #[serde(rename="id")]
    pub gid: Option<String>,
    #[serde(rename="inventoryItem")]
    pub inventory_item: Option<InventoryItemInput>,
    #[serde(rename="inventoryQuantities")]
    pub inventory_quantities: Option<Vec<InventoryLevelInput>>,
    #[serde(rename="mediaId")]
    pub media_gid: Option<String>,
    #[serde(rename="mediaSrc")]
    pub media_src: Option<String>,              // Vec of Strings (only one value)
    pub metafields: Option<Vec<MetafieldInput>>,
    pub options: Option<String>,                // Vec of Strings (like tags)
    pub position: Option<u32>,
    pub price: Option<f32>,                     // Money
    #[serde(rename="productId")]
    pub product_gid: Option<String>,
    #[serde(rename="requiresComponents")]
    pub requires_components: Option<bool>,
    #[serde(rename="taxCode")]
    pub tax_code: Option<String>,
    pub taxable: Option<bool>,
}

// Inventory Item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InventoryItemInput {
    pub cost: Option<String>,                // Decimal
    #[serde(rename="harmonizedSystemCode")]
    pub harmonized_system_code: Option<String>,
    pub measurement: Option<InventoryMeasurementInput>,
    #[serde(rename="requiresShipping")]
    pub requires_shipping: Option<bool>,
    #[serde(rename="tracked")]
    pub tracked: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InventoryMeasurementInput {
    pub weight: Option<WeightInput>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightInput {
    unit: WeightUnit
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WeightUnit {
    GRAMS,
    KILOGRAMS,
    OUNCES,
    POUNDS
}

impl WeightUnit {
    pub fn from_str(role: &str) -> WeightUnit {
        match role {
            "GRAMS" | _=> WeightUnit::GRAMS,              
            "KILOGRAMS" => WeightUnit::KILOGRAMS,
            "OUNCES" | _=> WeightUnit::OUNCES,
            "POUNDS" => WeightUnit::POUNDS,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            WeightUnit::GRAMS => String::from("GRAMS"),
            WeightUnit::KILOGRAMS => String::from("KILOGRAMS"),
            WeightUnit::OUNCES => String::from("OUNCES"),
            WeightUnit::POUNDS => String::from("POUNDS"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InventoryLevelInput {
    #[serde(rename="availableQuantity")]
    pub available_quantity: Option<u32>,
    #[serde(rename="locationId")]
    pub location_gid: Option<String>
}

// ** BASE PRODUCT STRUCT **
// https://shopify.dev/docs/api/admin-graphql/2025-01/objects/Product
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Product {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="id")]
    pub gid: Option<String>,
    pub title: Option<String>,
    pub metafields: Option<MetafieldConnection>,
    // ...
}