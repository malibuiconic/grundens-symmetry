use serde::{Deserialize, Serialize};
use crate::{AppError, RateLimiter, ShopifyCreds, PageInfo, shopify_graphql_request_with_retries};
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct OrderLineItemsResponse {
    order: OrderLineItems,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct OrderLineItems {
    #[serde(rename = "lineItems")]
    line_items: LineItemConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LineItemConnection {
    edges: Option<Vec<LineItemEdge>>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LineItemEdge {
    node: LineItem,
    cursor: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LineItem {
    id: String,
    title: String,
    #[serde(rename = "discountAllocations")]
    discount_allocations: Option<Vec<DiscountAllocation>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DiscountAllocation {
    #[serde(rename = "discountApplication")]
    discount_application: DiscountApplication,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "__typename")]
enum DiscountApplication {
    #[serde(rename = "DiscountCodeApplication")]
    DiscountCodeApplication { code: String },
    // You can add other variants here if needed
}

#[derive(Debug, Clone)]
pub struct LineItemDiscountInfo {
    pub line_item_id: String,
    pub line_item_title: String,
    pub applied_discount_codes: Vec<String>,
}

pub async fn get_order_line_item_discount_codes(
    shop: &ShopifyCreds,
    order_gid: &str,
    rate_limiter: &RateLimiter,
) -> Result<Vec<LineItemDiscountInfo>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mut results = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let after_param = match &cursor {
            Some(c) => format!(r#", after: "{}""#, c),
            None => String::new(),
        };

        let query = format!(
            r#"query getOrderLineItemDiscounts($orderId: ID!) {{
                order(id: $orderId) {{
                    lineItems(first: 100{after}) {{
                        edges {{
                            node {{
                                id
                                title
                                discountAllocations {{
                                    discountApplication {{
                                        __typename
                                        ... on DiscountCodeApplication {{
                                            code
                                        }}
                                    }}
                                }}
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
            after = after_param
        );

        let variables = json!({ "orderId": order_gid });

        let (response, _) = shopify_graphql_request_with_retries::<OrderLineItemsResponse, _>(
            gql_client.clone(),
            &query,
            Some(variables),
            3,
            rate_limiter,
        )
        .await
        .map_err(|e| AppError::OtherError(e.to_string()))?;

        if let Some(edges) = response.order.line_items.edges {
            for edge in edges {
                let line_item = edge.node;
                let mut codes = Vec::new();
                if let Some(allocs) = line_item.discount_allocations {
                    for alloc in allocs {
                        let DiscountApplication::DiscountCodeApplication { code } = alloc.discount_application;
                        codes.push(code);
                    }
                }

                results.push(LineItemDiscountInfo {
                    line_item_id: line_item.id,
                    line_item_title: line_item.title,
                    applied_discount_codes: codes,
                });

                cursor = Some(edge.cursor);
            }
        }

        let has_next = response
            .order
            .line_items
            .page_info
            .as_ref()
            .and_then(|p| p.has_next_page)
            .unwrap_or(false);

        if !has_next {
            break;
        }

        cursor = response
            .order
            .line_items
            .page_info
            .as_ref()
            .and_then(|p| p.end_cursor.clone());
    }

    Ok(results)
}
