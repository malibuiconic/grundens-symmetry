use crate::{ error::AppError, RateLimiter, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use serde_json::json;
use crate::PageInfo;
use crate::shopify_graphql_request_with_retries;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerPaymentMethod {
    pub id: String,
    pub last_digits: String,
    pub expiry_month: u32,
    pub expiry_year: u32,
    pub expires_soon: bool,
    pub source: String,     // Added source
    pub brand: String,      // Added brand
}

#[derive(Debug, Deserialize)]
struct PaymentMethodConnection {
    pub edges: Vec<PaymentMethodEdge>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct PaymentMethodEdge {
    pub node: PaymentMethodNode,
    pub cursor: String,
}

#[derive(Debug, Deserialize)]
struct PaymentMethodNode {
    pub id: String,
    pub instrument: PaymentInstrument,
}

#[derive(Debug, Deserialize)]
struct PaymentInstrument {
    #[serde(rename = "lastDigits")]
    pub last_digits: String,
    #[serde(rename = "expiryMonth")]
    pub expiry_month: u32,
    #[serde(rename = "expiryYear")]
    pub expiry_year: u32,
    #[serde(rename = "expiresSoon")]
    pub expires_soon: bool,
    pub source: String,     // Added source
    pub brand: String,      // Added brand
}

#[derive(Debug, Deserialize)]
struct CustomerData {
    customer: Customer,
}

#[derive(Debug, Deserialize)]
struct Customer {
    id: String,
    #[serde(rename = "paymentMethods")]
    payment_methods: PaymentMethodConnection,
}

pub async fn get_customer_payment_methods_by_id(
    shop: &ShopifyCreds,
    customer_id: &str,
    rate_limiter: &RateLimiter,
) -> Result<Vec<CustomerPaymentMethod>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mut all_payment_methods = Vec::new();
    let mut cursor: Option<String> = None;
    
    loop {
        let cursor_param = match &cursor {
            Some(c) => format!(", after: \"{}\"", c),
            None => String::new(),
        };

        let query = format!(
            r#"
            query {{
                customer(id: "{}") {{
                    id
                    paymentMethods(first: 10{}) {{
                        edges {{
                            node {{
                                id
                                instrument {{
                                    ... on CustomerCreditCard {{
                                        lastDigits
                                        expiryMonth
                                        expiryYear
                                        expiresSoon
                                        source
                                        brand
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
            }}
            "#,
            customer_id,
            cursor_param
        );

        match shopify_graphql_request_with_retries::<CustomerData, ()>(
            gql_client.clone(),
            &query,
            None,
            3,
            rate_limiter
        ).await {
            Ok((response, _extensions)) => {
                let payment_methods = response.customer.payment_methods;
                
                // Convert and add current page's payment methods to our collection
                for edge in payment_methods.edges {
                    let payment_method = CustomerPaymentMethod {
                        id: edge.node.id,
                        last_digits: edge.node.instrument.last_digits,
                        expiry_month: edge.node.instrument.expiry_month,
                        expiry_year: edge.node.instrument.expiry_year,
                        expires_soon: edge.node.instrument.expires_soon,
                        source: edge.node.instrument.source,      // Added source
                        brand: edge.node.instrument.brand,        // Added brand
                    };
                    all_payment_methods.push(payment_method);
                }

                // Check if we need to continue pagination
                if payment_methods.page_info.has_next_page.unwrap_or(false) {
                    cursor = payment_methods.page_info.end_cursor;
                } else {
                    break;
                }
            },
            Err(e) => {
                return Err(AppError::OtherError(e.to_string()));
            }
        }
        
        // Optional: Add a small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(all_payment_methods)
}