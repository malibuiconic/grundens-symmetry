use serde::{ Deserialize, Serialize };
use crate::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::products::OptionCreateInput;
use crate::error::UserError;
use crate::PageInfo;
use crate::variants::MetafieldConnection;
use crate::metafields::MetafieldEdge;
use crate::metafields::Metafield;
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantMetafieldsResponse {
    pub data: ProductVariantData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantData {
    #[serde(rename = "productVariant")]
    pub product_variant: ProductVariantMetafields,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantMetafields {
    pub id: String,
    pub metafields: MetafieldConnection,
}

pub async fn get_product_variant_metafields(
    shop: &ShopifyCreds,
    variant_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Vec<Metafield>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mut all_edges = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let after_param = match &cursor {
            Some(c) => format!(r#", after: "{}""#, c),
            None => String::new(),
        };

        let query = format!(
            r#"query getVariantMetafields($variantId: ID!) {{
                productVariant(id: $variantId) {{
                    id
                    metafields(first: 100{}) {{
                        edges {{
                            node {{
                                namespace
                                key
                                value
                            }}
                            cursor
                        }}
                        pageInfo {{
                            hasNextPage
                            endCursor
                        }}
                    }}
                }}
            }}"#,
            after_param
        );

        let variables = json!({
            "variantId": variant_gid,
        });

        match shopify_graphql_request_with_retries::<ProductVariantMetafieldsResponse, serde_json::Value>(
            gql_client.clone(),
            &query,
            Some(variables),
            3,
            rate_limiter,
        )
        .await
        {
            Ok((response, _)) => {
                let metafields = &response.data.product_variant.metafields;
                if let Some(edges) = &metafields.edges {
                    all_edges.extend(edges.clone());
                }

                if !metafields.page_info.as_ref().and_then(|pi| pi.has_next_page).unwrap_or(false) {
                    // Filter out None values and collect only the Some(Metafield) values
                    return Ok(all_edges
                        .into_iter()
                        .filter_map(|edge| edge.node)
                        .collect());
                }

                cursor = metafields
                    .page_info
                    .as_ref()
                    .and_then(|pi| pi.end_cursor.clone());
            }
            Err(e) => return Err(AppError::OtherError(e.to_string())),
        }
    }
}