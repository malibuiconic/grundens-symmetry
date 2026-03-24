use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────────────────────────────────────────
// getFulfillmentOrderByOrderId
// Before calling fulfillmentCreateV2, we must resolve the FulfillmentOrder GID
// from the Order GID.  Shopify's fulfillment API is order-line based, not
// order-level based from 2023 onwards.
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the first unfulfilled fulfillment-order GID for a given Shopify order GID.
pub async fn get_fulfillment_order_id(
    shop: &ShopifyCreds,
    shopify_order_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Option<String>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let query = format!(
        r#"query {{
            order(id: "{order_id}") {{
                fulfillmentOrders(first: 5) {{
                    edges {{
                        node {{
                            id
                            status
                        }}
                    }}
                }}
            }}
        }}"#,
        order_id = shopify_order_gid
    );

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, ()>(
        gql_client,
        &query,
        None,
        3,
        rate_limiter,
    )
    .await?;

    // Return the first OPEN fulfillment order
    let fulfillment_order_id = response
        .get("order")
        .and_then(|o| o.get("fulfillmentOrders"))
        .and_then(|fo| fo.get("edges"))
        .and_then(|e| e.as_array())
        .and_then(|arr| {
            arr.iter().find(|edge| {
                edge.get("node")
                    .and_then(|n| n.get("status"))
                    .and_then(|s| s.as_str())
                    .map(|s| s == "OPEN")
                    .unwrap_or(false)
            })
        })
        .and_then(|edge| edge.get("node"))
        .and_then(|node| node.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string());

    Ok(fulfillment_order_id)
}

// ──────────────────────────────────────────────────────────────────────────────
// fulfillmentCreateV2
// Creates a new fulfillment with optional tracking info for a fulfillment order.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/fulfillmentCreateV2
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentTrackingInput {
    pub company: Option<String>,
    pub number: Option<String>,
    pub url: Option<String>,
    pub numbers: Option<Vec<String>>,
    pub urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentOrderLineItemInput {
    pub id: String,       // Shopify FulfillmentOrderLineItem GID
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentOrderInput {
    #[serde(rename = "fulfillmentOrderId")]
    pub fulfillment_order_id: String,
    #[serde(rename = "fulfillmentOrderLineItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_order_line_items: Option<Vec<FulfillmentOrderLineItemInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentV2Input {
    #[serde(rename = "lineItemsByFulfillmentOrder")]
    pub line_items_by_fulfillment_order: Vec<FulfillmentOrderInput>,
    #[serde(rename = "notifyCustomer")]
    pub notify_customer: Option<bool>,
    #[serde(rename = "trackingInfo")]
    pub tracking_info: Option<FulfillmentTrackingInput>,
}

/// Creates a new Shopify fulfillment for an order.
/// Returns the new fulfillment GID.
pub async fn fulfillment_create_v2(
    shop: &ShopifyCreds,
    fulfillment_input: FulfillmentV2Input,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation fulfillmentCreateV2($fulfillment: FulfillmentV2Input!) {
            fulfillmentCreateV2(fulfillment: $fulfillment) {
                fulfillment {
                    id
                    status
                    trackingInfo {
                        company
                        number
                        url
                    }
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({ "fulfillment": fulfillment_input });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("fulfillmentCreateV2")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "fulfillmentCreateV2 userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("fulfillmentCreateV2")
        .and_then(|r| r.get("fulfillment"))
        .and_then(|f| f.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::OtherError("No fulfillment ID in fulfillmentCreateV2 response".to_string()))
}

// ──────────────────────────────────────────────────────────────────────────────
// getFulfillmentByOrderId
// Retrieves the first fulfillment GID for an order (needed for fulfillmentEventCreate).
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the first fulfillment GID on an order, or None.
pub async fn get_fulfillment_gid_by_order(
    shop: &ShopifyCreds,
    shopify_order_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Option<String>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let query = format!(
        r#"query {{
            order(id: "{order_id}") {{
                fulfillments(first: 5) {{
                    id
                    status
                }}
            }}
        }}"#,
        order_id = shopify_order_gid
    );

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, ()>(
        gql_client,
        &query,
        None,
        3,
        rate_limiter,
    )
    .await?;

    let fulfillment_id = response
        .get("order")
        .and_then(|o| o.get("fulfillments"))
        .and_then(|f| f.as_array())
        .and_then(|arr| arr.first())
        .and_then(|f| f.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string());

    Ok(fulfillment_id)
}

// ──────────────────────────────────────────────────────────────────────────────
// fulfillmentEventCreate
// Adds a fulfillment event (e.g. DELIVERED) to an existing fulfillment.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/fulfillmentEventCreate
// ──────────────────────────────────────────────────────────────────────────────

/// Attaches a delivery-status event to an existing fulfillment.
/// `fulfillment_gid` = gid://shopify/Fulfillment/...
/// `event_status`    = "DELIVERED" | "IN_TRANSIT" | "OUT_FOR_DELIVERY" | "CONFIRMED" etc.
/// `happened_at`     = ISO-8601 timestamp when the event occurred
pub async fn fulfillment_event_create(
    shop: &ShopifyCreds,
    fulfillment_gid: &str,
    event_status: &str,
    happened_at: Option<&str>,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation fulfillmentEventCreate($fulfillmentEvent: FulfillmentEventInput!) {
            fulfillmentEventCreate(fulfillmentEvent: $fulfillmentEvent) {
                fulfillmentEvent {
                    id
                    status
                    happenedAt
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let mut event_input = json!({
        "fulfillmentId": fulfillment_gid,
        "status": event_status,
    });

    if let Some(ts) = happened_at {
        event_input["happenedAt"] = json!(ts);
    }

    let variables = json!({ "fulfillmentEvent": event_input });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("fulfillmentEventCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "fulfillmentEventCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("fulfillmentEventCreate")
        .and_then(|r| r.get("fulfillmentEvent"))
        .and_then(|f| f.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::OtherError("No fulfillmentEvent ID in response".to_string()))
}
