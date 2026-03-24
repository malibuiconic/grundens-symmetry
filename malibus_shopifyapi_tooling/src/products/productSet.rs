use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use crate::metafields::MetafieldInput;
use serde::{Deserialize, Serialize};
use serde_json::json;

// https://shopify.dev/docs/api/admin-graphql/latest/mutations/productSet
// productSet is the idempotent upsert — preferred over productCreate for ERP sync.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductSetInput {
    /// When provided, Shopify will update the existing product instead of creating a new one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    #[serde(rename = "descriptionHtml")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(rename = "productType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    /// e.g. "ACTIVE" | "ARCHIVED" | "DRAFT"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Comma-separated tag string or individual tags (Shopify accepts both).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<SeoInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafields: Option<Vec<MetafieldInput>>,
    #[serde(rename = "productOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_options: Option<Vec<ProductOptionInput>>,
    /// Variant-level data included in the productSet payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<ProductSetVariantInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOptionInput {
    pub name: String,
    pub values: Vec<ProductOptionValueInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOptionValueInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductSetVariantInput {
    /// Shopify variant GID — omit for new variants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    pub price: String,
    #[serde(rename = "compareAtPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_at_price: Option<String>,
    pub taxable: Option<bool>,
    pub barcode: Option<String>,
    #[serde(rename = "inventoryItem")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_item: Option<VariantInventoryItemInput>,
    /// Option values that map to the product's productOptions array.
    #[serde(rename = "optionValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_values: Option<Vec<VariantOptionValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafields: Option<Vec<MetafieldInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInventoryItemInput {
    /// Whether Shopify tracks inventory for this variant.
    pub tracked: Option<bool>,
    #[serde(rename = "requiresShipping")]
    pub requires_shipping: Option<bool>,
    pub sku: Option<String>,
    pub measurement: Option<VariantMeasurementInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMeasurementInput {
    pub weight: Option<WeightInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightInput {
    pub value: f64,
    pub unit: String, // "POUNDS" | "KILOGRAMS" | "GRAMS" | "OUNCES"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantOptionValue {
    /// e.g. "Pack Size"
    #[serde(rename = "optionName")]
    pub option_name: String,
    /// e.g. "Case (10 Reams)"
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSetResult {
    /// Shopify product GID
    pub product_id: String,
    /// Vec of (sku, variant_gid, inventory_item_gid)
    pub variants: Vec<ProductSetVariantResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSetVariantResult {
    pub sku: Option<String>,
    pub variant_id: String,
    pub inventory_item_id: String,
}

/// Idempotent product upsert.
///
/// Set `input.id` to the existing Shopify product GID to update, or omit to create.
/// `synchronous: true` blocks until the operation is complete (required to get back IDs).
///
/// Returns a `ProductSetResult` with the product GID and a mapping of SKU → variant GID
/// and inventory item GID — store these in the DB for downstream inventory/price operations.
pub async fn product_set(
    shop: &ShopifyCreds,
    input: ProductSetInput,
    rate_limiter: &RateLimiter,
) -> Result<ProductSetResult, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation productSet($input: ProductSetInput!) {
            productSet(synchronous: true, input: $input) {
                product {
                    id
                    title
                    variants(first: 50) {
                        edges {
                            node {
                                id
                                sku
                                inventoryItem {
                                    id
                                }
                            }
                        }
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Vars {
        input: ProductSetInput,
    }

    let vars = Vars { input };

    let (response, _) =
        shopify_graphql_request_with_retries::<serde_json::Value, Vars>(
            gql_client,
            mutation,
            Some(vars),
            3,
            rate_limiter,
        )
        .await?;

    if let Some(errors) = response
        .get("productSet")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "productSet userErrors: {:?}",
                errors
            )));
        }
    }

    let product_id = response
        .get("productSet")
        .and_then(|r| r.get("product"))
        .and_then(|p| p.get("id"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| AppError::OtherError("No product ID in productSet response".to_string()))?
        .to_string();

    let variants = response
        .get("productSet")
        .and_then(|r| r.get("product"))
        .and_then(|p| p.get("variants"))
        .and_then(|v| v.get("edges"))
        .and_then(|e| e.as_array())
        .map(|edges| {
            edges
                .iter()
                .filter_map(|edge| {
                    let node = edge.get("node")?;
                    let variant_id = node.get("id")?.as_str()?.to_string();
                    let sku = node.get("sku").and_then(|s| s.as_str()).map(|s| s.to_string());
                    let inventory_item_id = node
                        .get("inventoryItem")
                        .and_then(|ii| ii.get("id"))
                        .and_then(|id| id.as_str())?
                        .to_string();
                    Some(ProductSetVariantResult {
                        sku,
                        variant_id,
                        inventory_item_id,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(ProductSetResult { product_id, variants })
}
