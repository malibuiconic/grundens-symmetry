use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::money::Decimal;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::{
    SubscriptionDraft, 
    SubscriptionLine,
    SubscriptionManualDiscount,
};

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/subscriptionDraftLineRemove
// has: The list of updated subscription discounts impacted by the removed line.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftLineResponse {
    #[serde(rename = "subscriptionDraftLineRemove")]
    pub subscription_draft_line_remove: Option<SubscriptionDraftLineRemove>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftLineRemove {
    pub draft: Option<SubscriptionDraft>,
    #[serde(rename="lineRemoved")]
    pub line_removed: Option<SubscriptionLine>,
    #[serde(rename="discountsUpdated")]
    pub discounts_updated: Option<Vec<SubscriptionManualDiscount>>,
    #[serde(rename="userErrors")]
    pub user_errors: Vec<UserError>
}

pub async fn remove_subscription_draft_line_input(
    shop: &ShopifyCreds,
    draft_id: &str,
    line_id: &str,
    rate_limiter: &RateLimiter
) -> Result<SubscriptionDraft, AppError> {

    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation subscriptionDraftLineRemove($draftId: ID!, $lineId: ID!){
            subscriptionDraftLineRemove(draftId: $draftId, lineId: $lineId){
                draft {
                    id
                    status
                }
                lineRemoved {
                    id
                    productId
                    quantity
                    sellingPlanId
                    sellingPlanName
                    variantId
                    title
                    variantTitle
                }    
                discountsUpdated {
                    id
                    title
                }
                userErrors{
                    field
                    message
                }    
            }
    }"#;

    let line_to_delete = serde_json::json!({"draftId": draft_id, "lineId": line_id});
        
    println!("Removing subscription draft line item: {}", line_id);

    match shopify_graphql_request_with_retries::<SubscriptionDraftLineResponse, serde_json::Value>(
        gql_client,
        mutation,
        Some(line_to_delete),
        3,
        rate_limiter
        ).await {
            Ok((response, _)) => {
                match response.subscription_draft_line_remove {
                    Some(subscription_draft_line) => {
                        if !subscription_draft_line.user_errors.is_empty(){
                            let error = format!("User errors: {:?}", subscription_draft_line.user_errors);
                            return Err(AppError::OtherError(error));
                        }
                        if let Some(draft) = subscription_draft_line.draft {
                            Ok(draft)
                        } else {
                            return Err(AppError::OtherError("Empty SubscriptionDraft - Draft line Removal".to_string()))
                        }
                    },
                    None => {
                        return Err(AppError::OtherError("Subscription Draft Line Remove - None Err".to_string()))
                    }
                }    
            },
            Err(e) => {
                println!("GraphQL request failed: {}", e);
                Err(AppError::OtherError(e.to_string()))
            }
    }
}

