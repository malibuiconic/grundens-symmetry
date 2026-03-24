use crate::{error::AppError, RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use crate::metafields::MetafieldInput;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────────────────────────────────────────
// companyCreate
// https://shopify.dev/docs/api/admin-graphql/latest/mutations/companyCreate
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyCreateInput {
    pub company: CompanyInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInput {
    pub name: String,
    #[serde(rename = "externalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafields: Option<Vec<MetafieldInput>>,
}

/// Creates a new B2B company and returns its Shopify GID.
pub async fn company_create(
    shop: &ShopifyCreds,
    input: CompanyCreateInput,
    rate_limiter: &RateLimiter,
) -> Result<String, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    let mutation = r#"
        mutation companyCreate($input: CompanyCreateInput!) {
            companyCreate(input: $input) {
                company {
                    id
                    name
                    externalId
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
        .get("companyCreate")
        .and_then(|r| r.get("userErrors"))
        .and_then(|e| e.as_array())
    {
        if !errors.is_empty() {
            return Err(AppError::OtherError(format!(
                "companyCreate userErrors: {:?}",
                errors
            )));
        }
    }

    response
        .get("companyCreate")
        .and_then(|r| r.get("company"))
        .and_then(|c| c.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::OtherError("No company ID in companyCreate response".to_string()))
}

// ──────────────────────────────────────────────────────────────────────────────
// getCompanyByExternalId
// Look up a company by its ERP external ID so we can upsert.
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the Shopify company GID if a company with the given `external_id` exists, else None.
pub async fn get_company_gid_by_external_id(
    shop: &ShopifyCreds,
    external_id: &str,
    rate_limiter: &RateLimiter,
) -> Result<Option<String>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);

    // Escape double-quotes in external_id to prevent query injection
    let safe_id = external_id.replace('"', "\\\"");
    let query = format!(
        r#"query {{
            companies(first: 1, query: "external_id:{external_id}") {{
                edges {{
                    node {{
                        id
                        externalId
                    }}
                }}
            }}
        }}"#,
        external_id = safe_id
    );

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, ()>(
        gql_client,
        &query,
        None,
        3,
        rate_limiter,
    )
    .await?;

    let company_gid = response
        .get("companies")
        .and_then(|c| c.get("edges"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first())
        .and_then(|edge| edge.get("node"))
        .and_then(|node| node.get("id"))
        .and_then(|id| id.as_str())
        .map(|id| id.to_string());

    Ok(company_gid)
}
