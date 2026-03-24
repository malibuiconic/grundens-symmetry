use serde_json::json;
use crate::AppError;
use crate::ShopifyCreds;
use crate::RateLimiter;
use crate::variants::ProductVariantsBulkInput;
use crate::shopify_graphql_request_with_retries;

pub async fn product_variants_bulk_update(
    shop: &ShopifyCreds,
    product_id: String,
    variants: Vec<ProductVariantsBulkInput>,
    rate_limiter: &RateLimiter,
) -> Result<(), AppError> {
    // Define the GraphQL mutation
    let mutation = r#"
        mutation productVariantsBulkUpdate($productId: ID!, $variants: [ProductVariantsBulkInput!]!) {
            productVariantsBulkUpdate(productId: $productId, variants: $variants) {
                productVariants {
                    id
                    title
                    price
                }
                userErrors {
                    field
                    message
                }
            }
        }
    "#;

    // Prepare the input variables for the mutation
    let variables = json!({
        "productId": product_id,
        "variants": variants,
    });

    // Execute the GraphQL request with retries
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3, // Number of retries
        rate_limiter,
    )
    .await
    .map_err(|e| AppError::OtherError(e.to_string()))?;

    // Check for user errors in the response
    if let Some(user_errors) = response.get("userErrors") {
        if !user_errors.is_null() {
            return Err(AppError::OtherError(format!(
                "Failed to update variants: {:?}",
                user_errors
            )));
        }
    }

    Ok(())
}
