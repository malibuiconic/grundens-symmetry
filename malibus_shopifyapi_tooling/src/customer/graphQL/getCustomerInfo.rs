use crate::{ error::AppError, ShopifyCreds};
use serde::{ Deserialize, Serialize };
use crate::error::UserError;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};

#[derive(Debug, Clone, Deserialize)]
struct CustomerResponse {
    customers: Option<CustomerEdge>,    
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerEdge {
    edges: Option<Vec<CustomerNode>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerNode {
    node: Option<Node>
}

#[derive(Debug, Clone, Deserialize)]
struct Node {
    id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerGIDResponse {
    pub customer_gid: Option<String>,
}

pub async fn get_customer_gid_by_email(shop: &ShopifyCreds, customer_email: &str, rate_limiter: &RateLimiter)
-> Result<CustomerGIDResponse, AppError>
{
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let query = format!("query {{
        customers(first: 1, query:\"{}\"){{
            edges {{
               node {{
                  id
               }}
            }}
        }}
    }}", customer_email);
    
    match shopify_graphql_request_with_retries::<CustomerResponse, ()>(gql_client, &query, None, 3, rate_limiter).await {
        Ok((response, _)) => {
            let customer_gid = response.customers
                .and_then(|customers| customers.edges)
                .and_then(|edges| edges.first().cloned())
                .and_then(|node| node.node)
                .and_then(|node| node.id);
            
            Ok(CustomerGIDResponse { customer_gid })
        },
        Err(e) => Err(AppError::OtherError(e.to_string()))
    }
}