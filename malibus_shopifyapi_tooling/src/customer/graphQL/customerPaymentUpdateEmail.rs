use crate::{ error::AppError, RateLimiter, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

pub async fn customer_payment_method_update_email(shop: &ShopifyCreds, customer_payment_method_id: &str, rate_limiter: &RateLimiter)
-> Result<(), AppError>{
    
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation 
    sendCustomerPaymentUpdateEmail($customerPaymentMethodId: ID!) {
        customerPaymentMethodSendUpdateEmail(customerPaymentMethodId: $customerPaymentMethodId) {
            customer {
                 id
            }
            userErrors {
                field
                message
            }
        }
    }"#;
    
    let customer_payment_update_email = json!({"customerPaymentMethodId": customer_payment_method_id});
    
    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(gql_client, &mutation, Some(customer_payment_update_email), 3, rate_limiter).await{
        Ok((response, exceptions)) => {
            println!("{:?}", response);
            Ok(())
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}