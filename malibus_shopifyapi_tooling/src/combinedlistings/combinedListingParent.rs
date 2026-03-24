#![allow(unused)]
use crate::{ error::AppError, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::error::UserError;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::products::ProductInput;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ProductCreateResponse {
   #[serde(rename="productCreate")]
   product_create: Option<NewProductCreated>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewProductCreated {
   product: Option<CombinedParentProduct>,
   #[serde(rename="userErrors")]
   user_errors: Option<Vec<UserError>>
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CombinedParentProduct{
   pub handle: Option<String>,
   pub title: Option<String>,
   pub id: Option<String>,
}


pub async fn create_combined_listing_parent(shop: &ShopifyCreds, product_input: &ProductInput, rate_limiter: &RateLimiter)
-> Result<CombinedParentProduct, AppError>{
    
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let mutation = r#"mutation
   productCreate($input: ProductInput!){
      productCreate(input: $input){
         product{
            id
            handle
         }
         userErrors{
            field
            message
         }
      }
   }"#;
   
   let new_combined_listing_parent = json!({ "input": product_input.clone() });
   // println!("{}",serde_json::to_string_pretty(&new_product).unwrap());
   match shopify_graphql_request_with_retries::<ProductCreateResponse, serde_json::Value>(gql_client, &mutation, Some(new_combined_listing_parent), 3, rate_limiter).await{
       Ok((response, extensions)) => {
        println!("Combined Listing Parent: {:?}", response);
        if let Some(product_create_response) = response.product_create {
            if let Some(new_product) = product_create_response.product {
               return Ok(new_product)
            }
          }
          return Ok(CombinedParentProduct::default())
       },
       Err(e) => { 
         Err(AppError::OtherError(e.to_string()))
      }
   }
}