use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::money::Decimal;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::{
    SubscriptionDraft, 
    SubscriptionPricingPolicyInput,
    Attribute,
};

// https://shopify.dev/docs/api/admin-graphql/unstable/mutations/subscriptiondraftlineupdate

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftLineAdd {
    pub current_price: Option<Decimal>,
    pub custom_attributes: Option<Vec<Attribute>>,
    pub pricing_policy: Option<SubscriptionPricingPolicyInput>,
    pub product_variant_id: Option<String>,
    pub quantity: Option<u32>,
    pub selling_plan_id: Option<String>,
    pub selling_plan_name: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftLineInput {
    #[serde(rename = "draftId")]
    pub draft_id: String,
    pub input: SubscriptionLineInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionLineInput {
    #[serde(rename = "currentPrice", skip_serializing_if = "Option::is_none")]
    pub current_price: Option<Decimal>,  
    #[serde(rename = "customAttributes", skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<Vec<Attribute>>,
    #[serde(rename = "pricingPolicy", skip_serializing_if = "Option::is_none")]
    pub pricing_policy: Option<SubscriptionPricingPolicyInput>,
    #[serde(rename = "productVariantId", skip_serializing_if = "Option::is_none")]
    pub product_variant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    #[serde(rename = "sellingPlanId", skip_serializing_if = "Option::is_none")]
    pub selling_plan_id: Option<String>,
    #[serde(rename = "sellingPlanName", skip_serializing_if = "Option::is_none")]
    pub selling_plan_name: Option<String>,
}

// Response types remain the same
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftLineResponse {
    #[serde(rename = "subscriptionDraftLineAdd")]
    pub subscription_contract_line_add: SubscriptionDraftLineAddResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDraftLineAddResponse {
    pub draft: SubscriptionDraft,
    #[serde(rename = "userErrors")]
    pub user_errors: Vec<UserError>,
}

pub async fn add_subscription_draft_line_input(
    shop: &ShopifyCreds,
    draft_id: &str,
    updates: SubscriptionDraftLineAdd,
    rate_limiter: &RateLimiter
) -> Result<SubscriptionDraft, AppError> {

    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation subscriptionDraftLineAdd($draftId: ID!, $input: SubscriptionLineInput!){
            subscriptionDraftLineAdd(draftId: $draftId, input: $input) {
                draft {
                    id
                    status
                    nextBillingDate
                    billingPolicy {
                        interval
                        intervalCount
                        minCycles
                        maxCycles
                    }
                    lines(first: 1){
                        edges {
                           node {
                              id
                              currentPrice {
                                amount
                              }
                              customAttributes {
                                key
                                value
                              }
                              productId
                              title
                              variantId
                              variantTitle
                              quantity
                              sellingPlanId
                           }
                        }
                    } 
                }
                lineAdded {
                    id 
                }
                userErrors {
                    field
                    message
                }        
            }
        }
    "#;
    // Validate quantity
    let quantity = updates.quantity.unwrap_or(1).max(1);

    let input = DraftLineInput {
        draft_id: draft_id.to_string(),
        input: SubscriptionLineInput {
            current_price: updates.current_price,
            custom_attributes: updates.custom_attributes,
            pricing_policy: updates.pricing_policy,
            product_variant_id: updates.product_variant_id,
            quantity: Some(quantity),
            selling_plan_id: updates.selling_plan_id,
            selling_plan_name: updates.selling_plan_name
        }
    };

    println!("Adding subscription draft line item with input: {:?}", input);

    match shopify_graphql_request_with_retries::<SubscriptionDraftLineResponse, DraftLineInput>(
        gql_client,
        mutation,
        Some(input),
        3,
        rate_limiter
    ).await {
        Ok((response, _)) => {
            if !response.subscription_contract_line_add.user_errors.is_empty() {
                let error = format!("User errors: {:?}", response.subscription_contract_line_add.user_errors);
                println!("{}", error);
                return Err(AppError::OtherError(error));
            }
            Ok(response.subscription_contract_line_add.draft)
        },
        Err(e) => {
            println!("GraphQL request failed: {}", e);
            Err(AppError::OtherError(e.to_string()))
        }
    }
}