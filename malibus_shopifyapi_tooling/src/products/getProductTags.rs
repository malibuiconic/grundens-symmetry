use serde::{ Deserialize, Serialize };
use crate::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::products::Product;
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

#[derive(Debug, Serialize, Deserialize)]
struct ProductTagsResponse {
    product: ProductTagsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductTagsData {
    id: String,
    tags: Vec<String>,
}

pub async fn get_product_tags(
    shop: &ShopifyCreds,
    product_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Vec<String>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let query = r#"query getProductTags($productId: ID!) {
            product(id: $productId) {
                id
                tags
            }
    }"#;

    let variables = json!({
        "productId": product_gid,
    });

    match shopify_graphql_request_with_retries::<ProductTagsResponse, serde_json::Value>(
        gql_client.clone(),
        &query,
        Some(variables),
        3,
        rate_limiter,
    ).await {
        Ok((response, _)) => {
           Ok(response.product.tags)
        }
        Err(e) => return Err(AppError::OtherError(e.to_string())),
    }
}
    