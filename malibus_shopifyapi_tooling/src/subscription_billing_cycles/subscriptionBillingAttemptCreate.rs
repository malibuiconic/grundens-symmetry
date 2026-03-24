use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use chrono::{ DateTime, Utc };
use crate::{shopify_graphql_request_with_retries, RateLimiter};

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/subscriptionbillingattemptcreate

pub async fn subscription_billing_cycle_attempt_create(
    shop: &ShopifyCreds,
    rate_limiter: &RateLimiter,
    contract_id: &str,
    index: i32,
    origin_time: &str,
    idempotency_key: &str,
) -> Result<(), AppError>{

    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"
        mutation subscriptionBillingAttemptCreate(
            $contractId: ID!,
            $index: Int!,
            $originTime: DateTime!,
            $idempotencyKey: String!
        ) {
            subscriptionBillingAttemptCreate(
                subscriptionContractId: $contractId,
                subscriptionBillingAttemptInput: {
                    billingCycleSelector: { index: $index },
                    idempotencyKey: $idempotencyKey,
                    originTime: $originTime
                }
            ) {
                subscriptionBillingAttempt {
                    id
                    ready
                }
                userErrors {
                    field
                    message
                }
            }
        }
    "#;
    
    let input = json!({
       "contractId": contract_id,
       "index": index,
       "originTime": origin_time,
       "idempotencyKey": idempotency_key
    });
    
    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        &mutation,
        Some(input),
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            println!("BS Attempt Charge Response: {:#?}", response);
            
            Ok(())
        },
        Err(e) => {
            println!("Billing Schedule Update Error: {:#?}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }

}