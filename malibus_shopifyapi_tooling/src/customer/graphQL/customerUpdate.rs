use crate::{ error::AppError, RateLimiter, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

use super::CustomerInput;

pub async fn customer_update(
    shop: &ShopifyCreds, 
    customer_input: &CustomerInput, 
    rate_limiter: &RateLimiter
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    
    let mutation = r#"
    mutation customerUpdate($input: CustomerInput!) {
        customerUpdate(input: $input) {
            customer {
                id
                metafields(first: 11) {
                    edges {
                        node {
                            id
                            namespace
                            key
                            value
                        }
                    }
                }
            }
            userErrors {
                message
                field
            }
        }
    }"#;

    let update_customer = json!({"input": customer_input});

    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(update_customer),
        3,
        rate_limiter
    ).await {
        Ok((response, exceptions)) => {
           // Check for user errors first
           if let Some(user_errors) = response
                .get("customerUpdate")
                .and_then(|customer_update| customer_update.get("userErrors"))
                .and_then(|errors| errors.as_array()) {
                
                if !user_errors.is_empty() {
                    println!("User Errors: {:?}", user_errors);
                    return Err(AppError::OtherError(format!(
                        "Failed to update customer: {:?}",
                        user_errors
                    )));
                }
            }

            // Get customer ID from the response
            let customer_id = response
                .get("customerUpdate")
                .and_then(|update| update.get("customer"))
                .and_then(|customer| customer.get("id"))
                .and_then(|id| id.as_str())
                .ok_or_else(|| {
                    println!("Full response: {}",
                        serde_json::to_string_pretty(&response).unwrap()
                    );
                    AppError::OtherError("No customer ID in response".to_string())
                })?;

            println!("Customer update cost: {:?}", exceptions.unwrap().cost);
            Ok(customer_id.to_string())
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}