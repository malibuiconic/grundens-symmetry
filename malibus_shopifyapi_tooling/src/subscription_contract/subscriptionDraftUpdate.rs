use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use chrono::{DateTime, Utc};
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::{
    SubscriptionDraft, 
    SubscriptionContractSubscriptionStatus,
    SubscriptionBillingPolicy,
    SubscriptionDeliveryMethod,
    SubscriptionDeliveryPolicy,
    Attribute,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftUpdate {
    pub billing_policy: Option<SubscriptionBillingPolicy>,
    pub custom_attributes: Option<Vec<Attribute>>,
    pub delivery_method: Option<SubscriptionDeliveryMethod>,
    pub delivery_policy: Option<SubscriptionDeliveryPolicy>,
    pub delivery_price: Option<String>,
    pub next_billing_date: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub payment_method_id: Option<String>,
    pub status: Option<SubscriptionContractSubscriptionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftUpdateInput {
    #[serde(rename = "draftId")]
    pub draft_id: String,
    pub input: SubscriptionDraftUpdateInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftUpdateInput {
    #[serde(rename = "billingPolicy", skip_serializing_if = "Option::is_none")]
    pub billing_policy: Option<SubscriptionBillingPolicy>,
    #[serde(rename = "customAttributes", skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<Vec<Attribute>>,
    #[serde(rename = "deliveryMethod", skip_serializing_if = "Option::is_none")]
    pub delivery_method: Option<SubscriptionDeliveryMethod>,
    #[serde(rename = "deliveryPolicy", skip_serializing_if = "Option::is_none")]
    pub delivery_policy: Option<SubscriptionDeliveryPolicy>,
    #[serde(rename = "deliveryPrice", skip_serializing_if = "Option::is_none")]
    pub delivery_price: Option<String>,
    #[serde(rename = "nextBillingDate", skip_serializing_if = "Option::is_none")]
    pub next_billing_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(rename = "paymentMethodId", skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SubscriptionContractSubscriptionStatus>,
}

// Response types remain the same
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftResponse {
    #[serde(rename = "subscriptionDraftUpdate")]
    pub subscription_contract_update: SubscriptionDraftUpdateResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftUpdateResponse {
    pub draft: SubscriptionDraft,
    #[serde(rename = "userErrors")]
    pub user_errors: Vec<UserError>,
}

// you can see a version of this operating here: src > webhook_handlers > shopifySubscriptionContractsWebhooks.rs scroll to bottom
// I used it in the updating next charge date. 
pub async fn update_subscription_draft(
    shop: &ShopifyCreds,
    draft_id: &str,
    updates: SubscriptionDraftUpdate,
    rate_limiter: &RateLimiter
) -> Result<SubscriptionDraft, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    
    let mutation = r#"
        mutation subscriptionDraftUpdate($draftId: ID!, $input: SubscriptionDraftInput!) {
            subscriptionDraftUpdate(draftId: $draftId, input: $input) {
                draft {
                    id
                    status
                    customer {
                        id
                    }
                    customerPaymentMethod {
                        id
                    }    
                    customAttributes{
                        key
                        value
                    }
                    nextBillingDate
                    note
                    billingPolicy {
                        interval
                        intervalCount
                        minCycles
                        maxCycles
                    }
                }
                userErrors {
                    field
                    message
                }
            }
        }
    "#;

    let input = DraftUpdateInput {
        draft_id: draft_id.to_string(),
        input: SubscriptionDraftUpdateInput {
            billing_policy: updates.billing_policy,
            custom_attributes: updates.custom_attributes,
            delivery_method: updates.delivery_method,
            delivery_policy: updates.delivery_policy,
            delivery_price: updates.delivery_price,
            next_billing_date: updates.next_billing_date,
            note: updates.note,
            payment_method_id: updates.payment_method_id,
            status: updates.status,
        },
    };

    println!("Updating subscription draft with input: {:?}", input);

    match shopify_graphql_request_with_retries::<SubscriptionDraftResponse, DraftUpdateInput>(
        gql_client,
        mutation,
        Some(input),
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            if !response.subscription_contract_update.user_errors.is_empty() {
                let error = format!("User errors: {:?}", response.subscription_contract_update.user_errors);
                println!("{}", error);
                return Err(AppError::OtherError(error));
            }
            Ok(response.subscription_contract_update.draft)
        },
        Err(e) => {
            println!("GraphQL request failed: {}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }
}