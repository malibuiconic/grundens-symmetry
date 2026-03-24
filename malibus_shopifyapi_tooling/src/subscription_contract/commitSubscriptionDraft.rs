use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::SubscriptionDraft;

pub async fn commit_subscription_draft(
    shop: &ShopifyCreds,
    draft_id: &str,
    rate_limiter: &RateLimiter
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"
        mutation subscriptionDraftCommit($draftId: ID!) {
            subscriptionDraftCommit(draftId: $draftId) {
                contract {
                    id
                    status
                    nextBillingDate
                    lastPaymentStatus
                    billingPolicy {
                        interval
                        intervalCount
                        minCycles
                        maxCycles
                    }
                    billingAttempts(first: 5) {
                        edges {
                            node {
                                id
                                completedAt
                                createdAt
                                processingError {
                                    code
                                    message
                                }
                            }
                        }
                    }
                    orders(first: 5) {
                        edges {
                            node {
                                id
                                createdAt
                            }
                        }
                    }
                }
                userErrors {
                    field
                    message
                }
            }
        }
    "#;

    let input = json!({
        "draftId": draft_id,
    });

    match shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        &mutation,
        Some(input),
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            println!("Commit Response: {:#?}", response);
            
            let contract_id = response
                .get("subscriptionDraftCommit")
                .and_then(|commit| commit.get("contract"))
                .and_then(|contract| contract.get("id"))
                .and_then(|id| id.as_str())
                .ok_or_else(|| AppError::OtherError("Failed to get contract ID".to_string()))?;

            Ok(contract_id.to_string())
        },
        Err(e) => {
            println!("Commit Error: {:#?}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }
}