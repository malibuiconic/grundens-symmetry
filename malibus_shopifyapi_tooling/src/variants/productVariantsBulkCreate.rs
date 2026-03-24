use crate::{error::AppError, ShopifyCreds};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::variants::ProductVariantsBulkInput;
use crate::{shopify_graphql_request_with_retries, RateLimiter};

pub async fn product_variants_bulk_create(
    shop: &ShopifyCreds,
    product_id: String,
    variants: Vec<ProductVariantsBulkInput>,
    rate_limiter: &RateLimiter,
) -> Result<Vec<String>, AppError> {
    let mutation = r#"
        mutation productVariantsBulkCreate($productId: ID!, $variants: [ProductVariantsBulkInput!]!) {
            productVariantsBulkCreate(productId: $productId, variants: $variants) {
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

    let variant_inputs: Vec<serde_json::Value> = variants.iter().map(|variant| {
        let option_values = variant.option_values.as_ref().map(|opts| 
            opts.iter().map(|opt| json!({
                "name": opt.name,
                "optionName": opt.option_name,
                "optionId": opt.option_id,
            })).collect::<Vec<_>>()
        ).unwrap_or_default();

        // Add metafields to the variant input
        let metafields = variant.metafields.as_ref().map(|metas| 
            metas.iter().map(|meta| json!({
                "key": meta.key,
                "namespace": meta.namespace,
                "type": "single_line_text_field",  // or the appropriate type
                "value": meta.value
            })).collect::<Vec<_>>()
        ).unwrap_or_default();

        json!({
            "price": variant.price,
            "optionValues": option_values,
            "metafields": metafields,  // Include metafields in the variant input
            "inventoryItem": {
                "requiresShipping": variant.inventory_item.as_ref().and_then(|item| item.requires_shipping).unwrap_or(false),
                "tracked": variant.inventory_item.as_ref().and_then(|item| item.tracked).unwrap_or(false)
            }
        })
    }).collect();
    
    let variables = json!({
        "productId": product_id,
        "variants": variant_inputs,
    });
    
    // priintln!("Full variables: {:?}", variables);

    let (response, _) = shopify_graphql_request_with_retries::<serde_json::Value, serde_json::Value>(
        ShopifyCreds::gql_client_builder(shop),
        mutation,
        Some(variables),
        3,
        rate_limiter,
    ).await.map_err(|e| AppError::OtherError(e.to_string()))?;

    // println!("Full GraphQL response: {:?}", response);

    // Check for user errors
    if let Some(user_errors) = response
        .get("productVariantsBulkCreate")
        .and_then(|bulk_create| bulk_create.get("userErrors")) {
        if let Some(errors) = user_errors.as_array() {
            if !errors.is_empty() {
                return Err(AppError::OtherError(format!("GraphQL errors: {:?}", errors)));
            }
        }
    }

    // Extract variant IDs from the response
    let variant_ids = response
        .get("productVariantsBulkCreate")
        .and_then(|bulk_create| bulk_create.get("productVariants"))
        .and_then(|variants| variants.as_array())
        .ok_or_else(|| AppError::OtherError("No variants returned".to_string()))?
        .iter()
        .filter_map(|variant| {
            variant.get("id")
                .and_then(|id| id.as_str())
                .map(String::from)
        })
        .collect::<Vec<String>>();

    if variant_ids.is_empty() {
        return Err(AppError::OtherError("No variants returned".to_string()));
    }

    Ok(variant_ids)
}
