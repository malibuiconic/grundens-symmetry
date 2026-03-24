use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::SubscriptionDraft;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Input {
    #[serde(rename="contractId")]
    contract_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionDraftResponse {
    #[serde(rename="subscriptionContractUpdate")]
    subscription_contract_update: SubscriptionContractUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionContractUpdate {
    draft: SubscriptionDraft,
    #[serde(rename="userErrors")]
    user_errors: Vec<UserError>,
}

// https://shopify.dev/docs/api/admin-graphql/2025-01/mutations/subscriptioncontractupdate
// 1 - create a draft of the contract you are interested in updating. Shopify provides this mutation
// 2 - update the draft (allows you to review in a draft state without messing with the live version)
// 3 - commit the draft to make changes perminent.
pub async fn subscription_contract_draft_create(shop: &ShopifyCreds, contract_id: &str, rate_limiter: &RateLimiter)
-> Result<SubscriptionDraft, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    subscriptionContractUpdate($contractId: ID!){
        subscriptionContractUpdate(contractId: $contractId){
           draft {
              id
              nextBillingDate
              note
              status
              billingPolicy{
                interval
                intervalCount
              }
              lines(first:1){
                  edges {
                     node {
                        id
                        currentPrice{
                            amount
                            currencyCode
                        }
                        customAttributes {
                            key 
                            value
                        }
                        productId
                        variantId
                        sellingPlanId
                        sellingPlanName
                        title
                        quantity
                        variantTitle          
                     }
                    cursor 
                  }
                  pageInfo{
                     hasNextPage
                     endCursor
                  }  
              }  
           }
           userErrors {
              field
              message
           }        
        }
    }"#;

    let new_subscription_contract_update = Input { 
        contract_id: contract_id.to_string(), 
    };
    
    match shopify_graphql_request_with_retries::<SubscriptionDraftResponse, Input>(gql_client, &mutation, Some(new_subscription_contract_update), 3, rate_limiter).await {
        Ok((response, _)) => {
            if !response.subscription_contract_update.user_errors.is_empty() {
                let error = format!("{:?}",response.subscription_contract_update.user_errors);
                return Err(AppError::OtherError(error.to_string())); 
            };
            Ok(response.subscription_contract_update.draft)
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}




