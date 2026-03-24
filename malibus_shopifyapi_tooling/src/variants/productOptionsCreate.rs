use serde::{ Deserialize, Serialize };
use crate::AppError;
use crate::RateLimiter;
use crate::ShopifyCreds;
use crate::products::OptionCreateInput;
use crate::error::UserError;
use serde_json::json;
use crate::shopify_graphql_request_with_retries;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductOptionsCreateResponse { // Make this struct public
    #[serde(rename="productOptionsCreate")]
    pub product_options_create: Option<ProductOptionsCreate>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductOptionsCreate { // Make this struct public
    pub product: Option<Product>, // Make this field public
    #[serde(rename="userErrors")]
    pub user_errors: Option<Vec<UserError>>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Product { // Make this struct public
    pub id: Option<String>, // Make this field public
    pub handle: Option<String>, // Make this field public
    pub variants: Option<Variants>, // Make this field public
    pub options: Option<Vec<ProductOption>>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductOption { // Make this struct public
    pub id: Option<String>, // Make this field public
    pub name: Option<String>, // Make this field public
    #[serde(rename="optionValues")]
    pub option_values: Option<Vec<OptionValue>>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionValue { // Make this struct public
    pub id: Option<String>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Variants { // Make this struct public
    pub nodes: Option<Vec<Node>>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node { // Make this struct public
    pub id: Option<String>, // Make this field public
    #[serde(rename="selectedOptions")]
    pub selected_options: Option<Vec<SelectedOption>>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedOption { // Make this struct public
    pub name: Option<String>, // Make this field public
    pub value: Option<String>, // Make this field public
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LeadVariantOptionResponse { // Make this struct public
    pub product_id: Option<String>, // Make this field public
    pub variant_id: Option<String>, // Make this field public
    pub variant_url: Option<String>, // Make this field public
    pub selected_option: Option<Vec<SelectedOption>>, // Make this field public
    pub msg: Option<String>, // Make this field public
}

pub async fn product_options_create(shop: &ShopifyCreds, product_options: Vec<OptionCreateInput>, product_gid: &str, rate_limiter: &RateLimiter)
-> Result<ProductOptionsCreateResponse, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    createOptions($productId: ID!, $options: [OptionCreateInput!]!){
        productOptionsCreate(productId: $productId, options: $options){
            product {
                id
                handle
                variants(first: 1) {
                    nodes {
                        id
                        title
                        selectedOptions {
                            name
                            value
                        }
                    }
                }
                options {
                    id
                    name
                    values
                    position
                    optionValues {
                        id
                        name
                    }
                }
            }
            userErrors {
                field
                message
            }
        }
    }"#;
    let new_options = json!({"productId": product_gid, "options": product_options});
    match shopify_graphql_request_with_retries::<ProductOptionsCreateResponse, serde_json::Value>(gql_client, &mutation, Some(new_options), 3, rate_limiter).await {
        Ok((response, _)) => Ok(response),
        Err(e) => Err(AppError::OtherError(e.to_string())),
    }
}