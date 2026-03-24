use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::SubscriptionContract;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionContractResponse {
    #[serde(rename="subscriptionContract")]
    subscription_contract: serde_json::Value,
}

// https://shopify.dev/docs/api/admin-graphql/2025-01/queries/subscriptioncontract?language=graphql
pub async fn get_subscription_contract_by_id(
    shop: &ShopifyCreds,
    contract_id: &str,
    rate_limiter: &RateLimiter
) -> Result<SubscriptionContract, AppError>{
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let query = format!(r#"query 
        findContract($subscriptionContractId: ID!){{
            subscriptionContract(id: $subscriptionContractId){{
                id
                status
                customerPaymentMethod{{
                    id
                    instrument{{
                        __typename
                        ...on CustomerCreditCard{{
                            lastDigits
                            brand
                            name
                            source
                        }}
                    }}
                }}
                nextBillingDate
                customAttributes {{
                    key
                    value
                }}
                note
                linesCount {{
                    count
                }}
                billingPolicy {{
                    interval
                    intervalCount
                    maxCycles
                    minCycles
                    anchors {{
                       cutoffDay
                       day
                       month
                       type
                    }}
                }}
                createdAt
                customer {{
                    id
                    tags
                }}
                lines(first: 5){{
                    edges {{
                        node {{
                           id
                           productId
                           quantity
                           variantId
                           title
                           variantTitle
                           customAttributes {{
                                key
                                value
                           }}
                           lineDiscountedPrice {{
                                amount
                           }}
                           pricingPolicy {{
                                basePrice {{
                                    amount
                                }}
                                cycleDiscounts {{
                                    adjustmentType
                                    adjustmentValue
                                    afterCycle
                                    computedPrice {{
                                        amount
                                    }}
                                }}    
                           }}     
                        }}
                        cursor
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}    
                }}        
            }}
        }}
    "#);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Input {
        #[serde(rename="subscriptionContractId")]
        subscription_contract_id: String,
    }
    
    let subscription_contract = Input { subscription_contract_id: contract_id.to_string() };

    match shopify_graphql_request_with_retries::<SubscriptionContractResponse, Input>(gql_client, &query, Some(subscription_contract), 3, rate_limiter).await {
        Ok((response, _)) => {
            match serde_json::from_value::<SubscriptionContract>(response.subscription_contract) {
                Ok(subscription_contract) => {
                    Ok(subscription_contract)
                },
                Err(e) => {
                    Err(AppError::OtherError(format!("Failed to deserialize SubscriptionContract: {}", e)))
                }
            }
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}


// https://shopify.dev/docs/api/admin-graphql/2025-01/queries/subscriptioncontracts
pub async fn get_all_subscription_contracts(
    shop: &ShopifyCreds,
    rate_limiter: &RateLimiter
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let query = r#"{
        subscriptionContracts(first: 1, query: "status:ACTIVE" reverse: true) {
            edges {
                node {
                    id
                    status
                    nextBillingDate
                    customAttributes {
                       key
                       value
                    }
                }
                cursor
            }
            pageInfo {
               hasNextPage
               endCursor
            }
        }
    }"#;
    
    match shopify_graphql_request_with_retries::<serde_json::Value, ()>(
        gql_client,
        &query,
        None,
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            // println!("First 10, Contracts: {:#?}", response);
            Ok(())
        },
        Err(e) => {
            // println!("Query Error: {:#?}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }
}