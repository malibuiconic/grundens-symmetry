use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use chrono::{DateTime, Utc};
use crate::money::Decimal;
use crate::money::CurrencyCode;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::subscriptions::SubscriptionDraft;
use crate::subscriptions::SubscriptionContractSubscriptionStatus;
use crate::subscriptions::SubscriptionContractCreateInput;
use crate::subscriptions::SubscriptionContract;
use crate::subscriptions::SubscriptionDraftInput;
use crate::subscriptions::SubscriptionBillingPolicyInput;
use crate::selling_plans::SellingPlanAnchorInput;
use crate::subscriptions::Attribute;
use crate::subscriptions::SubscriptionLine;
use crate::selling_plans::SellingPlanInterval;
use crate::subscriptions::SubscriptionDeliveryPolicyInput;
use crate::selling_plans::SellingPlanAnchor;
use crate::selling_plans::SellingPlanAnchorType;


#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionDraftResponse {
    #[serde(rename="subscriptionContractCreate")]
    subscription_contract_create: SubscriptionContractCreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionContractCreate {
    draft: SubscriptionDraft,
    #[serde(rename="userErrors")]
    user_errors: Vec<UserError>,
}

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/subscriptionContractCreate
// This functionality will be used in the application for our purposes of pre-purchasing
// Say you want to start a subscription that could happen in the future. This is how we would
// do this. Or say you had an active sub for a customer, and you wanted to create a new sub for them
// outside of their actions. (Tom Ferry does this with renewals. As a sub may not be done (12 months),
// but at 90 days out they initiate a renewal that will start when the other sub ends if that makes sense ;)
pub async fn subscription_contract_create(shop: &ShopifyCreds, input: &SubscriptionContractCreateInput, rate_limiter: &RateLimiter)
-> Result<SubscriptionDraft, AppError>{
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    // add more return fields in the mutation here if you are looking to see
    // more info about the subscription draft returned ;) ie. billingPolicy, etc..
    let mutation = r#"mutation
    createSubscriptionContract($input: SubscriptionContractCreateInput!){
        subscriptionContractCreate(input: $input){
            draft{
               id
               customer {
                  id
               }
               customerPaymentMethod{
                  id
               }
            }
            userErrors {
               field
               message
            }
        }
    }    
    "#;

    let new_subscription_contract = serde_json::json!({"input": input});
    match shopify_graphql_request_with_retries::<SubscriptionDraftResponse, serde_json::Value>(gql_client, &mutation, Some(new_subscription_contract), 3, rate_limiter).await {
        Ok((response, _)) => {
            if !response.subscription_contract_create.user_errors.is_empty() {
                let error = format!("{:?}",response.subscription_contract_create.user_errors);
                return Err(AppError::OtherError(error.to_string())); 
            };
            // println!("New Contract Draft: {:?}", response.subscription_contract_create.draft);
            Ok(response.subscription_contract_create.draft)
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}


// For Renewals: Taking a current SubcriptionContract and making a copy (with some modifications)
// into a SubscriptionContractInput for use with SubscriptionContractCreate above.
// https://shopify.dev/docs/api/admin-graphql/2024-10/input-objects/SubscriptionContractCreateInput

pub async fn convert_active_sub_contract_to_renewal_sub_contract_input(
    shop: &ShopifyCreds, 
    rate_limiter: &RateLimiter, 
    active_contract: &SubscriptionContract, 
    renewal_start_date: Option<DateTime<Utc>>,
)
-> Result<SubscriptionDraft, AppError>{

    // Add validation for renewal_start_date
    let valid_renewal_date = match renewal_start_date {
        Some(date) => {
            let now = Utc::now();
            // If date is today, add one day to make it tomorrow
            if date.date_naive() == now.date_naive() {
                Some(now + chrono::Duration::days(1))
            } else {
                Some(date)
            }
        },
        None => Some(Utc::now() + chrono::Duration::days(1)) // Default to tomorrow if no date provided
    };
   
    // Convert Billing Policy as applicable
    // Now would we change this later depending on cadence inputs from the new sub we want? TODO!
    let billing_policy_input = active_contract.billing_policy.as_ref().map(|policy| {
        let anchors = policy.anchors.as_ref().map(|anchors| {
            anchors.iter()
                .filter_map(|anchor| {
                    anchor.selling_plan_anchor_type.as_ref().map(|anchor_type| {
                        SellingPlanAnchorInput {
                            cut_off_day: anchor.cut_off_day.clone(),
                            day: anchor.day.clone(),
                            month: anchor.month.clone(),
                            selling_plan_anchor_type: anchor_type.clone(),
                        }
                    })
                })
                .collect::<Vec<_>>()
        });
        SubscriptionBillingPolicyInput {
            anchors,
            interval: Some(SellingPlanInterval::MONTH),    // may need to make dynamic for future cadence changes
            interval_count: Some(12),
            max_cycles: policy.max_cycles.clone(),
            min_cycles: policy.min_cycles.clone(),
            ..Default::default()
        }
    });

    // Handle custom attributes
    let custom_attributes = active_contract.custom_attributes.as_ref().map(|attrs| {
        attrs.iter()
            .filter_map(|attr| {
                // Only include attributes where both key and value are present
                if let (Some(key), Some(value)) = (attr.key.as_ref(), attr.value.as_ref()) {
                    Some(Attribute {
                        key: Some(key.clone()),
                        value: Some(value.clone()),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }).unwrap_or_default();
    
    // Extract the Lines and edit as required.
    let lines = if let Some(line_connection) = &active_contract.lines {
        if let Some(edges) = &line_connection.edges {
            edges.iter()
                .filter_map(|edge| {
                    edge.node.as_ref().map(|line| {
                        SubscriptionLine{
                          product_id: line.product_id.clone(),
                          variant_id: line.variant_id.clone(),  
                          ..Default::default()  
                        }
                    })
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Customer Id associated with the Contract
    let customer_id = active_contract.customer.as_ref().and_then(|customer| customer.id.clone());
    
    // Customer Payment Method (which would be our tokenized card essentially)
    let payment_method_id = active_contract.customer_payment_method.as_ref().and_then(|method| method.id.clone());
    
    // Create Subscription Draft Input (Setting Up The Contract and Draft)
    // We will set some default behaviors for what a subscription draft will be set as.
    let subscription_draft_input = SubscriptionDraftInput {
        billing_policy: Some(SubscriptionBillingPolicyInput{
            min_cycles: Some(3),
            max_cycles: Some(12),
            interval: Some(SellingPlanInterval::MONTH),
            interval_count: Some(12),
            anchors: Some(vec![]),
            /*
            anchors: Some(vec![
                SellingPlanAnchorInput {
                    selling_plan_anchor_type: SellingPlanAnchorType::MONTHDAY,
                    day: Some(12),
                    cut_off_day: None,
                    month: None,      
                }
            ]),*/
        }),
        custom_attributes: Some(custom_attributes),
        delivery_method: None,
        delivery_policy: Some(SubscriptionDeliveryPolicyInput{
            interval: Some(SellingPlanInterval::MONTH),
            interval_count: Some(12),
            anchors: Some(vec![]),
        }),
        delivery_price: None,
        next_billing_date: valid_renewal_date,
        note: Some("Renewal Opportunity".to_string()),
        payment_method_id,
        status: Some(SubscriptionContractSubscriptionStatus::ACTIVE),
    };

    let subscription_contract_create_input = SubscriptionContractCreateInput {
        contract: Some(subscription_draft_input),
        currency_code: Some(CurrencyCode::USD),
        customer_id,
        next_billing_date: valid_renewal_date,
    };

    match subscription_contract_create(shop, &subscription_contract_create_input, rate_limiter).await{
        Ok(mut renewal_subscription_draft) => {
            // Once we have our Draft, we will manipulate our draft with the inputs of this function
            // Then once everything is golden, we will commit this subscription for the future!
            // println!("New Subscription Draft ID: {:?}", renewal_subscription_draft.id);
            Ok(renewal_subscription_draft)
        },
        Err(e) => { Err(AppError::OtherError(e.to_string())) }
    }
}

