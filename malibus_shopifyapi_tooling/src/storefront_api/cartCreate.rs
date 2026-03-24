use serde::{ Deserialize, Serialize };
use crate::error::AppError;
use crate::storefront_api::CartInput;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::shopify_graphql_request_with_retries;

// https://shopify.dev/docs/api/storefront/2025-01/mutations/cartCreate
#[derive(Debug, Deserialize)]
struct CartCreateResponse {
   cartCreate: CartCreateResult,
}

#[derive(Debug, Deserialize)]
struct CartCreateResult {
   cart: Option<CartInfo>,
   userErrors: Vec<UserError>, 
}

#[derive(Debug, Deserialize)]
struct CartInfo {
   id: String,
   checkoutUrl: String,
}

#[derive(Debug, Deserialize)]
struct UserError {
   field: Vec<String>,
   message: String,
}

pub async fn cart_create<'a>(cart_input: CartInput, shop: ShopifyCreds, rate_limiter: &'static RateLimiter) -> Result<String, AppError> {
   let gql_client = ShopifyCreds::gql_client_storefront_builder(&shop);
   let mutation = r#"mutation cartCreate($input: CartInput!) {
       cartCreate(input: $input) {
           cart {
               id
               checkoutUrl
           }
           userErrors {
               field
               message
           }
       }
   }"#;
   
   let new_cart = serde_json::json! ({ "input": cart_input });
   
   match shopify_graphql_request_with_retries::<CartCreateResponse, serde_json::Value>(gql_client, &mutation, Some(new_cart), 3, rate_limiter).await {
    Ok((response, extensions)) => {
        match response.cartCreate.cart {
            Some(cart) => Ok(cart.checkoutUrl),
            None => {
                let errors = response.cartCreate.userErrors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(AppError::OtherError(errors))
            }
        }
    },
    Err(e) => Err(AppError::OtherError(e.to_string()))
}
}