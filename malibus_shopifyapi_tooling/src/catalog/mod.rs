use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────────────────────────────────────────
// catalogCreate
// Creates a B2B company-scoped catalog and links it to a company.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/catalogCreate
// ──────────────────────────────────────────────────────────────────────────────

/// Creates a B2B catalog scoped to a single company and returns the new catalog GID.
/// `company_gid` = gid://shopify/Company/...
/// `title`       = human-readable catalog name (e.g. "Acme Corp — FY2026 Pricing")
/// Returns the catalog GID on success.
pub async fn catalog_create(
    shop: &ShopifyCreds,
    company_gid: &str,
    title: &str,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation catalogCreate($input: CatalogCreateInput!) {
            catalogCreate(input: $input) {
                catalog {
                    id
                    title
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({
        "input": {
            "title": title,
            "status": "ACTIVE",
            "context": {
                "companyLocationIds": [],  // we'll assign location in the next step
                "companyIds": [company_gid]
            }
        }
    });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("catalogCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "catalogCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("catalogCreate")
        .and_then(|r| r.get("catalog"))
        .and_then(|c| c.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::OtherError("No catalog ID in catalogCreate response".to_string()))
}

// ──────────────────────────────────────────────────────────────────────────────
// priceListCreate
// Creates a price list linked to a catalog.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/priceListCreate
// ──────────────────────────────────────────────────────────────────────────────

/// Creates a price list for a catalog and returns the new price list GID.
/// `catalog_gid` = gid://shopify/Catalog/...
/// `currency`    = ISO currency code, e.g. "USD"
pub async fn price_list_create(
    shop: &ShopifyCreds,
    catalog_gid: &str,
    name: &str,
    currency: &str,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation priceListCreate($input: PriceListCreateInput!) {
            priceListCreate(input: $input) {
                priceList {
                    id
                    name
                }
                userErrors {
                    field
                    message
                    code
                }
            }
        }
    "#;

    let variables = json!({
        "input": {
            "name": name,
            "currency": currency,
            "catalogId": catalog_gid,
            "parent": {
                "adjustment": {
                    "type": "PERCENTAGE_DECREASE",
                    "value": 0.0
                }
            }
        }
    });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("priceListCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "priceListCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("priceListCreate")
        .and_then(|r| r.get("priceList"))
        .and_then(|pl| pl.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::OtherError("No priceList ID in priceListCreate response".to_string()))
}

// ──────────────────────────────────────────────────────────────────────────────
// priceListFixedPricesAdd
// Adds fixed override prices for specific variants to a price list.
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/priceListFixedPricesAdd
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceListPriceInput {
    /// Shopify variant GID
    #[serde(rename = "variantId")]
    pub variant_id: String,
    pub price: MoneyInput,
    #[serde(rename = "compareAtPrice")]
    pub compare_at_price: Option<MoneyInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyInput {
    pub amount: String,   // decimal string e.g. "38.99"
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
}

/// Adds fixed override prices to an existing price list.
/// Each entry in `prices` maps a variant GID to a specific contracted price.
pub async fn price_list_fixed_prices_add(
    shop: &ShopifyCreds,
    price_list_id: &str,
    prices: Vec<PriceListPriceInput>,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation priceListFixedPricesAdd($priceListId: ID!, $prices: [PriceListPriceInput!]!) {
            priceListFixedPricesAdd(priceListId: $priceListId, prices: $prices) {
                prices {
                    variant {
                        id
                    }
                    price {
                        amount
                        currencyCode
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

    let variables = json!({
        "priceListId": price_list_id,
        "prices": prices,
    });

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3,
        rate_limiter,
    )
    .await?;

    if let Some(errors) = response
        .get("priceListFixedPricesAdd")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "priceListFixedPricesAdd userErrors: {:?}",
                errors
            )));
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// getPriceListByCompany
// Finds the price list GID associated with a company's catalog.
// Uses the catalogs query filtered by company.
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the price list GID for the first catalog linked to a company, or None if none exists.
pub async fn get_price_list_gid_by_company(
    shop: &ShopifyCreds,
    company_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Option<String>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    // Encode the company GID for the query filter
    let query = format!(
        r#"query {{
            catalogs(first: 1, query: "context_type:company context_id:{company_gid}") {{
                edges {{
                    node {{
                        id
                        priceList {{
                            id
                        }}
                    }}
                }}
            }}
        }}"#,
        company_gid = company_gid.replace('/', "\\/")
    );

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, ()>(
        gql_client,
        &query,
        None,
        3,
        rate_limiter,
    )
    .await?;

    let price_list_id = response
        .get("catalogs")
        .and_then(|c| c.get("edges"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first())
        .and_then(|edge| edge.get("node"))
        .and_then(|node| node.get("priceList"))
        .and_then(|pl| pl.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string());

    Ok(price_list_id)
}
