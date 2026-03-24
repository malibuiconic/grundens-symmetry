use serde::{ Deserialize, Serialize };
use crate::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::products::OptionCreateInput;
use crate::error::UserError;
use crate::PageInfo;
use crate::products::Product;
use crate::metafields::MetafieldEdge;
use crate::metafields::Metafield;
use serde_json::json;
use crate::shopify_graphql_request_with_retries;
use crate::products::MetafieldConnection;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductVariantMetafieldsResponse {
    pub product: ProductMetafields,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductMetafields {
    pub id: String,
    pub metafields: MetafieldConnection,
}

pub async fn get_product_metafields(
    shop: &ShopifyCreds,
    product_gid: &str,
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
            r#"query getProductMetafields($productId: ID!) {{
                product(id: $productId) {{
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
            "productId": product_gid,
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
                let metafields = &response.product.metafields;
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