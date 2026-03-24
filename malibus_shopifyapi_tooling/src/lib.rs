#![allow(unused)]
use crate::error::AppError;
use std::fmt::Debug;
use std::fmt;
use std::collections::HashMap;
use gql_client::Client;
use http::Extensions;
use serde_json::Value;
use serde::{ Deserialize, Serialize };
use tokio::time::{ sleep, Duration, Instant };
use tokio::sync::Mutex;

pub mod error;
pub mod shop;
pub mod app;
pub mod count;
pub mod address;
pub mod discount;
pub mod metaobjectdefinitions;
pub mod metaobject;
pub mod metafielddefinitions;
pub mod metafields;
pub mod products;
pub mod publishable_publish;
pub mod variants;
pub mod money;
pub mod image;
pub mod customer;
pub mod orders;
pub mod selling_plans;
pub mod subscription_contract;
pub mod subscription_billing_cycles;
pub mod subscriptions;
pub mod draft_orders;
pub mod payments;
pub mod company;
pub mod taxes;
pub mod inventory;   // inventorySetQuantities + inventoryAdjustQuantities
pub mod catalog;     // catalogCreate, priceListCreate, priceListFixedPricesAdd
pub mod fulfillment; // fulfillmentCreateV2, fulfillmentEventCreate
pub mod shipping;
pub mod user;  
pub mod storefront_api;
pub mod refund;
pub mod location;
pub mod r#return;
             
// Custom GQL Client Builder
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ShopifyCreds {
    pub shopify_token: String,
    pub shopify_api_version: String,
    pub myshopify_url: String,
    pub storefront_access_token: String,
}

impl ShopifyCreds {
    pub fn gql_client_builder(&self) -> Client {
        let mut headers = HashMap::new();
        headers.insert("X-Shopify-Access-Token", self.shopify_token.clone());
        headers.insert("X-GraphQL-Cost-Include-Fields", "true".to_string());
        headers.insert("Content-Type", "application/json".to_string());
        let query_url = format!("{}/admin/api/{}/graphql.json", self.myshopify_url, self.shopify_api_version);
        let gql_client = Client::new_with_headers(query_url, headers);
        gql_client
    }

    pub fn gql_client_storefront_builder(&self) -> Client {
        let mut headers = HashMap::new();
        headers.insert("X-Shopify-Storefront-Access-Token", self.storefront_access_token.clone());
        headers.insert("Content-Type", "application/json".to_string());
        
        let query_url = format!("{}/api/2025-01/graphql.json", self.myshopify_url);
        Client::new_with_headers(query_url, headers)
    }
}

// NOTE: This custom lib uses the gql_client crate. Which abstacts away additional metadata
// like extensions that Shopify would send back. Where it only handles "data" and "errors"
// So I had to fork the original and create some additional types to handle the Shopify extensions 
// Which you will see in the cargo.toml file that being the case, and the edits to the below code
// namely query_vars_with_extensions functionality. So unique to Shopify. I did this so I can 
// create a monitor to adhere to Shopify's rate limits on mutations and quries as indicated here:
// https://shopify.dev/docs/api/usage/rate-limits#cost-calculation

/*
   Shopify Retry and Latency Request(s) Wrapper Function

   This function allows for calls made to the Shopify system can
   include retries with an expoential backoff for when the network (Internet)
   is acting unstable for whatever reason.

   Set as a generic to allow for any queries to be flexible in their
   return types ;)
*/
#[derive(Debug, Deserialize)]
pub struct ShopifyExtensions {
    pub cost: CostInfo,
}

#[derive(Debug, Deserialize)]
pub struct CostInfo {
    #[serde(rename="actualQueryCost")]
    pub actual_query_cost: i32,
    #[serde(rename="requestedQueryCost")]
    pub requested_query_cost: i32,
    #[serde(rename="throttleStatus")]
    pub throttle_status: ThrottleStatus,
    pub fields: Vec<FieldCost>,
}

#[derive(Debug, Deserialize)]
pub struct ThrottleStatus {
    #[serde(rename="currentlyAvailable")]
    pub currently_available: f64,
    #[serde(rename="maximumAvailable")]
    pub maximum_available: f64,
    #[serde(rename="restoreRate")]
    pub restore_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct FieldCost {
    #[serde(rename="definedCost")]
    pub defined_cost: i32,
    pub path: Vec<String>,
    #[serde(rename="requestedChildrenCost")]
    pub requested_children_cost: Option<i32>,
    #[serde(rename="requestedTotalCost")]
    pub requested_total_cost: i32,
}

// Use serde's field renaming to match JSON keys
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedExtensions {
    pub cost: ParsedCost,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedCost {
    pub actual_query_cost: i32,
    pub requested_query_cost: i32,
    pub throttle_status: ThrottleStatus,
}

pub async fn shopify_graphql_request_with_retries<T, V>(
    mut gql_client: Client,
    query: &str,
    mutation_variables: Option<V>,
    max_retries: u32,
    rate_limiter: &RateLimiter
) -> Result<(T, Option<ParsedExtensions>), AppError>
where
    T: Debug + for<'de> serde::Deserialize<'de>,
    V: Debug + Clone + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    let mut retries_counter = 0;

    match mutation_variables {
        Some(variables) => {
            // Are we dealing with a GraphQL Mutation?. So Variables should be present!
            loop {
                // Wait for rate limit before making the request
                rate_limiter.wait(10).await?; // Assume a default cost of 10

                match gql_client.query_with_extensions::<T,V>(query, variables.clone()).await {
                    Ok(response) => {
                        let parsed_extensions = if let Some(shopify_extensions) = response.extensions {
                            match serde_json::from_value::<ParsedExtensions>(shopify_extensions.clone()) {
                                Ok(parsed) => {
                                    // Update the rate limiter with the latest Shopify throttle status
                                    rate_limiter.update_shopify_status(ShopifyThrottleStatus {
                                        maximum_available: parsed.cost.throttle_status.maximum_available,
                                        currently_available: parsed.cost.throttle_status.currently_available,
                                        restore_rate: parsed.cost.throttle_status.restore_rate,
                                    }).await;
                                    
                                    // Visual representation of current rate limit status
                                    display_rate_limit_status(&parsed);

                                    Some(parsed)
                                },
                                Err(e) => {
                                    println!("Failed to parse extensions: {:?}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        match response.data {
                            Some(resp) => {
                                return Ok((resp, parsed_extensions));
                            },
                            None => {
                                let error_msg = "GraphQL Client Mutation Response Error";
                                return Err(AppError::OtherError(error_msg.to_string()));
                            }
                        }    
                    },
                    Err(e) if retries_counter < max_retries => {
                        retries_counter += 1;
                        let backoff_time = 2u64.pow(retries_counter as u32) * 1000; 
                        // println!("Graphql Error:{}", e);
                        println!("Retrying Shopify GraphQL Mutation request after {} milliseconds", backoff_time);
                        sleep(Duration::from_millis(backoff_time)).await;
                    },
                    Err(err) => {
                        println!("{}", err);
                        return Err(AppError::OtherError(err.to_string()));
                    },
                }
            }
        },
        None => {
            // Are we dealing with just a GraphQL Query?. (Typically Doesn't have variables)
            loop {
                // Wait for rate limit before making the request
                rate_limiter.wait(10).await?; // Assume a default cost of 10

                match gql_client.query_with_extensions::<T, ()>(query, ()).await {
                    Ok(response) => {
                        let parsed_extensions = if let Some(shopify_extensions) = response.extensions {
                            match serde_json::from_value::<ParsedExtensions>(shopify_extensions.clone()) {
                                Ok(parsed) => {
                                    // Update the rate limiter with the latest Shopify throttle status
                                    rate_limiter.update_shopify_status(ShopifyThrottleStatus {
                                        maximum_available: parsed.cost.throttle_status.maximum_available,
                                        currently_available: parsed.cost.throttle_status.currently_available,
                                        restore_rate: parsed.cost.throttle_status.restore_rate,
                                    }).await;
                                    
                                    // Visual representation of current rate limit status
                                    display_rate_limit_status(&parsed);

                                    Some(parsed)
                                },
                                Err(e) => {
                                    println!("Failed to parse extensions: {:?}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        match response.data {
                            Some(resp) => {
                                return Ok((resp, parsed_extensions));
                            },
                            None => {
                                let error_msg = "GraphQL Client Query Response Error";
                                return Err(AppError::OtherError(error_msg.to_string()));
                            }
                        }    
                    },
                    Err(e) if retries_counter < max_retries => {
                        retries_counter += 1;
                        let backoff_time = 2u64.pow(retries_counter as u32) * 1000; 
                        // println!("Graphql Error:{}", e);
                        println!("Retrying Shopify GraphQL request after {} milliseconds", backoff_time);
                        sleep(Duration::from_millis(backoff_time)).await;
                    }
                    Err(err) => {
                        println!("{}", err);
                        return Err(AppError::OtherError(err.to_string()));
                    },
                }
            }
        }
    }
}

// Helper function to display rate limit status
fn display_rate_limit_status(parsed: &ParsedExtensions) {
    let percentage_available = (parsed.cost.throttle_status.currently_available / parsed.cost.throttle_status.maximum_available) * 100.0;
    let bar_length = 20;
    let filled_length = (percentage_available / 100.0 * bar_length as f64) as usize;
    let bar = format!(
        "[{}{}] {:.2}% ({:.2}/{:.2})",
        "=".repeat(filled_length),
        " ".repeat(bar_length - filled_length),
        percentage_available,
        parsed.cost.throttle_status.currently_available,
        parsed.cost.throttle_status.maximum_available
    );

    // println!("Rate Limit Status: {}", bar);
    // println!("Actual Query Cost: {}", parsed.cost.actual_query_cost);
    // println!("Restore Rate: {:.2} points/second", parsed.cost.throttle_status.restore_rate);
}

// RateLimiter Constructs - As Shopify depending on plan has Thresholds in Place
// https://shopify.dev/docs/api/usage/rate-limits

// GRAPHQL RATES
pub const SHOPIFY_ENTERPRISE_GRAPHQL_RATE: usize = 2000;
pub const SHOPIFY_PLUS_GRAPHQL_RATE: usize = 1000;               // points per second      
pub const SHOPIFY_ADVANCED_GRAPHQL_RATE: usize = 200;
pub const SHOPIFY_STANDARD_GRAPHQL_RATE: usize = 100;

// REST RATES
pub const SHOPIFY_ENTERPRISE_REST_RATE: usize = 40;
pub const SHOPIFY_PLUS_REST_RATE: usize = 20;                    // requests per second
pub const SHOPIFY_ADVANCED_REST_RATE: usize = 4;
pub const SHOPIFY_STANDARD_REST_RATE: usize = 2;

// STOREFRONT API
// NONE

// PAYMENTS APPS API (GRAPHQL)
pub const SHOPIFY_ENTERPRISE_PAYMENTS_RATE: usize = 3640;
pub const SHOPIFY_PLUS_PAYMENTS_RATE: usize = 1820;              // points per second
pub const SHOPIFY_ADVANCED_PAYMENTS_RATE: usize = 910;
pub const SHOPIFY_STANDARD_PAYMENTS_RATE: usize = 910;

// CUSTOMER ACCOUNT API
pub const SHOPIFY_ENTERPRISE_CUSTOMER_RATE: usize = 400;
pub const SHOPIFY_PLUS_CUSTOMER_RATE: usize = 200;               // points per second
pub const SHOPIFY_ADVANCED_CUSTOMER_RATE: usize = 200;
pub const SHOPIFY_STANDARD_CUSTOMER_RATE: usize = 100;

#[derive(Debug, Clone)]
pub struct ShopifyThrottleStatus {
    pub maximum_available: f64,
    pub currently_available: f64,
    pub restore_rate: f64,
}
#[derive(Debug)]
pub struct RateLimiter {
    pub points_per_second: usize,
    pub interval: Duration,
    pub last_check: Mutex<Instant>,
    pub tokens: Mutex<usize>,
    pub shopify_status: Mutex<Option<ShopifyThrottleStatus>>,
}

impl RateLimiter {
    pub fn new(points_per_second: usize, interval: Duration) -> Self {
        RateLimiter {
            points_per_second,
            interval,
            last_check: Mutex::new(Instant::now()),
            tokens: Mutex::new(points_per_second),
            shopify_status: Mutex::new(None),
        }
    }

    pub async fn update_shopify_status(&self, status: ShopifyThrottleStatus) {
        let mut shopify_status = self.shopify_status.lock().await;
        *shopify_status = Some(status);
    }

    pub async fn wait(&self, request_cost: usize) -> Result<(), AppError> {
        let mut last_check = self.last_check.lock().await;
        let mut tokens = self.tokens.lock().await;
        let shopify_status = self.shopify_status.lock().await;

        let now = Instant::now();
        let time_passed = now.duration_since(*last_check);

        if time_passed >= self.interval {
            *tokens = self.points_per_second;
            *last_check = now;
        }

        // Check Shopify's throttle status if available
        if let Some(status) = &*shopify_status {
            if status.currently_available < request_cost as f64 {
                let wait_time = (request_cost as f64 - status.currently_available) / status.restore_rate;
                tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
            }
        }

        if *tokens < request_cost {
            let sleep_duration = self.interval - time_passed;
            tokio::time::sleep(sleep_duration).await;
            *tokens = self.points_per_second;
            *last_check = Instant::now();
        }

        if *tokens < request_cost {
            return Err(AppError::OtherError("API rate limit Exceeded".to_string()));
        }

        *tokens -= request_cost;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageInfo {
    #[serde(rename="hasNextPage")]
    pub has_next_page: Option<bool>,
    #[serde(rename="endCursor")]
    pub end_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeInput {
    pub key: String,           // required when used
    pub value: String,         // required when used
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSON(pub Value);

impl JSON {
    // Create a new JSON from a serde_json::Value
    pub fn new(value: Value) -> Self {
        JSON(value)
    }

    // Parse from a JSON string
    pub fn parse(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s).map(JSON)
    }

    // Get the inner Value
    pub fn into_inner(self) -> Value {
        self.0
    }

    // Get a reference to the inner Value
    pub fn as_value(&self) -> &Value {
        &self.0
    }

    // Convert to a pretty-printed JSON string
    pub fn to_pretty_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.0)
    }
}

// Display implementation for easy printing
impl fmt::Display for JSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(&self.0) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "Invalid JSON"),
        }
    }
}

// Allow converting from serde_json::Value
impl From<Value> for JSON {
    fn from(value: Value) -> Self {
        JSON(value)
    }
}

// Allow converting to serde_json::Value
impl From<JSON> for Value {
    fn from(json: JSON) -> Self {
        json.0
    }
}
