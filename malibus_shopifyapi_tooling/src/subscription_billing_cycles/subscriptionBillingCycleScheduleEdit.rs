use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use chrono::{ DateTime, Utc };
use crate::{shopify_graphql_request_with_retries, RateLimiter};

pub async fn subscription_billing_cycle_schedule_edit(
    shop: &ShopifyCreds,
    rate_limiter: &RateLimiter,
    contract_id: &str,
    index: i32,
    date: DateTime<Utc>,
) -> Result<(), AppError>{
    
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"
        mutation subscriptionBillingCycleScheduleEdit($contractId: ID!, $index: Int!, $date: DateTime!){
            subscriptionBillingCycleScheduleEdit(billingCycleInput: {
            contractId: $contractId, selector: {index: $index}},
            input: {billingDate: $date, reason: DEV_INITIATED}) {
                billingCycle {
                    cycleIndex
                    billingAttemptExpectedDate
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
        "date": date,
    });

    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        &mutation,
        Some(input),
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            println!("BS Schedule Response: {:#?}", response);
            
            Ok(())
        },
        Err(e) => {
            println!("Billing Schedule Update Error: {:#?}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }
}