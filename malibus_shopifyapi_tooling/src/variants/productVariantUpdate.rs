use serde::{Deserialize, Serialize};
use crate::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::money::Money;
use crate::products::ProductVariantInput;
use crate::error::UserError;
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

////**** DEPREICATED *** Use bulk


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantUpdateResponse {
    #[serde(rename = "productVariantUpdate")]
    pub product_variant_update: Option<ProductVariantUpdate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantUpdate {
    #[serde(rename = "productVariant")]
    pub product_variant: Option<Variant>,
    #[serde(rename = "userErrors")]
    pub user_errors: Option<Vec<UserError>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Variant {
    pub id: Option<String>,
    pub title: Option<String>,
    pub price: Option<Money>,
    #[serde(rename = "compareAtPrice")]
    pub compare_at_price: Option<Money>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VariantUpdateResponse {
    pub id: Option<String>,
    pub title: Option<String>,
}

pub async fn product_variant_update(
    shop: &ShopifyCreds,
    variant_input: ProductVariantInput,
    rate_limiter: &RateLimiter,
) -> Result<VariantUpdateResponse, AppError> {
    // Validate the input (e.g., ensure `id` is present)
    if variant_input.gid.is_none() {
        return Err(AppError::OtherError("Variant ID is required".to_string()));
    }

    // Define the GraphQL mutation
    let mutation = r#"
        mutation productVariantUpdate($input: ProductVariantInput!) {
            productVariantUpdate(input: $input) {
                productVariant {
                    id
                    title
                    price
                    compareAtPrice
                }
                userErrors {
                    field
                    message
                }
            }
        }
    "#;

    // Prepare the input variables for the mutation
    let variables = json!({ "input": variant_input });

    // Execute the GraphQL request with retries
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let (response, _) = shopify_graphql_request_with_retries::<ProductVariantUpdateResponse, serde_json::Value>(
        gql_client,
        mutation,
        Some(variables),
        3, // Number of retries
        rate_limiter,
    )
    .await
    .map_err(|e| AppError::OtherError(e.to_string()))?;

    // Check for user errors in the response
    if let Some(product_variant_update) = &response.product_variant_update {
        if let Some(user_errors) = &product_variant_update.user_errors {
            if !user_errors.is_empty() {
                return Err(AppError::OtherError(format!(
                    "Failed to update variant: {:?}",
                    user_errors
                )));
            }
        }

        // Extract the updated variant details
        if let Some(product_variant) = &product_variant_update.product_variant {
            Ok(VariantUpdateResponse {
                id: product_variant.id.clone(),
                title: product_variant.title.clone(),
            })
        } else {
            Err(AppError::OtherError("No product variant found in the response".to_string()))
        }
    } else {
        Err(AppError::OtherError("No product variant update found in the response".to_string()))
    }
}