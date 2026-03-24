use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::refund::RefundInput;
use crate::refund::Refund;
use crate::orders::Order;
use crate::{shopify_graphql_request_with_retries, RateLimiter};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RefundCreateResponse {
    data: Option<RefundCreateData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RefundCreateData {
    #[serde(rename = "refundCreate")]
    refund_create: Option<RefundCreatePayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RefundCreatePayload {
    refund: Option<Refund>,
    user_errors: Vec<UserError>,
}

pub async fn refund_create(shop: &ShopifyCreds, refund_input: &RefundInput, rate_limiter: &RateLimiter)
-> Result<(Option<String>, Option<String>), AppError>{
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    refundCreate($input: RefundInput!){
       refundCreate(input: $input){
           refund {
              id
              note
              totalRefundedSet {
                presentmentMoney {
                    amount
                }
              }
           }
           userErrors {
                field
                message
           }   
       }
    }"#;

    let refund_input = serde_json::json!({"input": refund_input});

    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(gql_client, &mutation, Some(refund_input), 3, rate_limiter).await {
        Ok((response, _)) => {
            
            println!("{:?}", response);
            
            /*
            // Extract the refund ID and note from the response
            if let Some(data) = response.data {
                if let Some(refund_create) = data.refund_create {
                    if let Some(refund) = refund_create.refund {
                        return Ok((refund.id, refund.note));
                    }
                }
            }*/
            // If we get here, the response didn't contain the expected data
            Ok((None, None))
            // Err(AppError::OtherError("Failed to parse refund response".to_string()))
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}