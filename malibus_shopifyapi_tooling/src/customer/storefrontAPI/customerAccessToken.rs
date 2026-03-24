use crate::{ error::AppError, RateLimiter, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

use super::CustomerCreateInput;

pub async fn customer_create(shop: &ShopifyCreds, customer_input: &CustomerCreateInput, rate_limiter: &RateLimiter)
-> Result<(), AppError>{
    
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    customerCreate($input: CustomerCreateInput!){
       customerCreate(input: $input){
           customer {
               id
               email
               createdAt
           }
           customerUserErrors {
               field
               message
               code
           }
       }
    }
    "#;
    let new_customer = json!({"input": customer_input});
    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(gql_client, &mutation, Some(new_customer), 3, rate_limiter).await{
        Ok((response, exceptions)) => {
            Ok(())
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}