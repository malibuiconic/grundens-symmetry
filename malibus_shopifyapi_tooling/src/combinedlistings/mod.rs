#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };

// https://shopify.dev/docs/apps/build/product-merchandising/combined-listings/build-for-combined-listings
// Only available to Plus Shops!
// When you have more than 100 variants, and you aren't using the future 2K variants
// Uses the "UNSTABLE" GRAPHQL API for the products type, productCreate and combinedListingsUpdate mutations
// You already have products in your store you want to combine ;) 
// - Honestly this was a quick fix for large merchants when there wasn't/isn't the 2k variant option avail 
// - This will likely get deprecated at some point 

pub mod combinedListingParent;
pub mod addChildProductsToCombinedListing;
pub mod combinedListingRole;
pub mod combinedListingUpdate;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChildProductRelationInput {
    #[serde(rename="childProductId")]                                 // Required 
    pub child_product_id: String,                                     // This will be the Product GID of the variant (product) you want to combine
    #[serde(rename="selectedParentOptionValues")]                     // Required
    pub selected_parent_option_values: Vec<SelectedVariantOptionInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedVariantOptionInput {
    pub name: String,                       // required
    pub value: String,                      // required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="linkedMetafieldValue")]
    pub linked_metafield_value: Option<String>  // this could be a color metaobject ID for example
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionsAndValues {
    pub name: String,         // for instance "color"
    pub values: Vec<String>,  // vec of String values (tied to SelectedVariantOptionInput)
}
