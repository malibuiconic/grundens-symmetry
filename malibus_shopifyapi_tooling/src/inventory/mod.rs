use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────────────────────────────────────────
// inventorySetQuantities
// Use when update_type == "set" (absolute / cycle-count driven)
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/inventorySetQuantities
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySetQuantityInput {
    /// Shopify inventoryItem GID (gid://shopify/InventoryItem/...)
    #[serde(rename = "inventoryItemId")]
    pub inventory_item_id: String,
    /// Shopify location GID (gid://shopify/Location/...)
    #[serde(rename = "locationId")]
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySetQuantitiesInput {
    pub reason: String,             // e.g. "correction", "received", "damaged"
    #[serde(rename = "referenceDocumentUri")]
    pub reference_document_uri: Option<String>,
    pub quantities: Vec<InventorySetQuantityInput>,
    #[serde(rename = "ignoreCompareQuantity")]
    pub ignore_compare_quantity: Option<bool>,
    #[serde(rename = "setOnHand")]
    pub set_on_hand: Option<bool>,
}

/// Sets inventory to an absolute quantity.
/// `reason` maps from ERP reasons → Shopify reason slugs:
///   "CORRECTION" → "correction"
///   "RECEIVED"   → "received"
///   "DAMAGED"    → "damaged"
///   "SOLD"       → "correction" (use delta for sales; fall back to set)
///   "RETURNED"   → "received"
pub async fn inventory_set_quantities(
    shop: &ShopifyCreds,
    input: InventorySetQuantitiesInput,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation inventorySetQuantities($input: InventorySetQuantitiesInput!) {
            inventorySetQuantities(input: $input) {
                inventoryAdjustmentGroup {
                    id
                    reason
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({ "input": input });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("inventorySetQuantities")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "inventorySetQuantities userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// inventoryAdjustQuantities
// Use when update_type == "delta" (relative change)
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/inventoryAdjustQuantities
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAdjustQuantityInput {
    #[serde(rename = "inventoryItemId")]
    pub inventory_item_id: String,
    #[serde(rename = "locationId")]
    pub location_id: String,
    pub delta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAdjustQuantitiesInput {
    pub reason: String,
    #[serde(rename = "referenceDocumentUri")]
    pub reference_document_uri: Option<String>,
    pub changes: Vec<InventoryAdjustQuantityInput>,
    pub name: String, // "available"
}

/// Adjusts inventory by a relative delta.
/// `name` should always be "available" for stock adjustments.
pub async fn inventory_adjust_quantities(
    shop: &ShopifyCreds,
    input: InventoryAdjustQuantitiesInput,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation inventoryAdjustQuantities($input: InventoryAdjustQuantitiesInput!) {
            inventoryAdjustQuantities(input: $input) {
                inventoryAdjustmentGroup {
                    id
                    reason
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({ "input": input });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("inventoryAdjustQuantities")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "inventoryAdjustQuantities userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}

/// Maps an ERP reason string to the Shopify inventory reason slug.
pub fn erp_reason_to_shopify(reason: &str) -> String {
    match reason.to_uppercase().as_str() {
        "CORRECTION" => "correction".to_string(),
        "RECEIVED"   => "received".to_string(),
        "DAMAGED"    => "damaged".to_string(),
        "SOLD"       => "correction".to_string(),
        "RETURNED"   => "received".to_string(),
        _            => "correction".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::erp_reason_to_shopify;

    #[test]
    fn maps_correction() {
        assert_eq!(erp_reason_to_shopify("CORRECTION"), "correction");
    }

    #[test]
    fn maps_received() {
        assert_eq!(erp_reason_to_shopify("RECEIVED"), "received");
    }

    #[test]
    fn maps_damaged() {
        assert_eq!(erp_reason_to_shopify("DAMAGED"), "damaged");
    }

    #[test]
    fn sold_maps_to_correction() {
        // SOLD uses inventoryAdjustQuantities (delta); if used in set context falls back
        assert_eq!(erp_reason_to_shopify("SOLD"), "correction");
    }

    #[test]
    fn returned_maps_to_received() {
        assert_eq!(erp_reason_to_shopify("RETURNED"), "received");
    }

    #[test]
    fn unknown_reason_defaults_to_correction() {
        assert_eq!(erp_reason_to_shopify("GARBAGE"),    "correction");
        assert_eq!(erp_reason_to_shopify(""),            "correction");
        assert_eq!(erp_reason_to_shopify("WRITE_OFF"),  "correction");
    }

    #[test]
    fn input_is_case_insensitive() {
        assert_eq!(erp_reason_to_shopify("correction"), "correction");
        assert_eq!(erp_reason_to_shopify("Received"),   "received");
        assert_eq!(erp_reason_to_shopify("dAmAgEd"),    "damaged");
    }
}
