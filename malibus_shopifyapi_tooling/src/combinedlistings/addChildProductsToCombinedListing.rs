#![allow(unused)]
use crate::{ error::AppError, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::error::UserError;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::combinedlistings::{ ChildProductRelationInput, OptionsAndValues };

// https://shopify.dev/docs/apps/build/product-merchandising/combined-listings/build-for-combined-listings

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CombinedListingResponse {
    #[serde(rename="combinedListingUpdate")]
    combined_listing_update: Option<CombinedListingUpdate>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CombinedListingUpdate {
    product: Option<Product>,
    userErrors: Option<Vec<UserError>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Product {
    #[serde(rename="combinedListingRole")]
    combined_listing_role: Option<String>,   // Swap to future Enum
    #[serde(rename="combinedListing")]
    combined_listing: Option<serde_json::Value>,
    // The other stuff..
}


pub async fn add_child_products_to_combined_listing(shop: &ShopifyCreds, combined_listing_parent_gid: String, products_added: Vec<ChildProductRelationInput>, option_and_values: Vec<OptionsAndValues>, rate_limiter: &RateLimiter)
-> Result<(), AppError>{

   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let mutation = r#"mutation
     AddChildProductsToCombinedListing($parentProductId: ID!, $productsAdded: [ChildProductRelationInput!], $optionsAndValues: [OptionAndValueInput!]) {
         combinedListingUpdate(parentProductId: $parentProductId, productsAdded: $productsAdded, optionsAndValues: $optionsAndValues) {
             product {
                id
                combinedListing {
                    combinedListingChildren(first: 20) {
                    nodes {
                        product {
                        id
                        }
                        parentVariant {
                        selectedOptions {
                            value
                        }
                        }
                    }
                }
            }
        }
        userErrors {
            code
            field
            message
        }
     }
   }"#;

   let combined_childern = json!({ "parentProductId": combined_listing_parent_gid, "productsAdded": products_added, "optionsAndValues": option_and_values });
   // println!("{}", serde_json::to_string_pretty(&combined_childern).unwrap());
   match shopify_graphql_request_with_retries::<CombinedListingResponse, serde_json::Value>(gql_client, &mutation, Some(combined_childern), 3, rate_limiter).await{
        Ok((response, extensions)) => {
        
        if let Some(combined_response) = response.combined_listing_update {
            if let Some(user_errors) = combined_response.userErrors {
                println!("Errors with Combined Children: {:?}", user_errors)
            }
        }

        // will update later
        return Ok(())
        },
        Err(e) => { 
             Err(AppError::OtherError(e.to_string()))
        }
    }
}