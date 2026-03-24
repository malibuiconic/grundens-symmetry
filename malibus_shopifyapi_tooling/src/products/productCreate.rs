use crate::{error::AppError, ShopifyCreds};
use crate::products::ProductInput;
use serde::{Deserialize, Serialize};
use crate::error::UserError;
use serde_json::json;
use crate::{shopify_graphql_request_with_retries, RateLimiter};
use crate::publishable_publish::publications::get_publications;
use crate::publishable_publish::PublicationInput;
use crate::publishable_publish::publishablePublish::publishable_publish;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductCreateResponse {
    #[serde(rename = "productCreate")]
    pub product_create: Option<NewProductCreated>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewProductCreated {
    pub product: Option<ProductInfo>,
    #[serde(rename = "userErrors")]
    pub user_errors: Option<Vec<UserError>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductInfo {
    pub handle: Option<String>,
    pub id: Option<String>,
    pub variants: Option<Variants>, // Add variants field to the response
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Variants {
    pub nodes: Option<Vec<VariantNode>>, // Add nodes field for variants
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariantNode {
    pub id: Option<String>,     // Variant ID
    pub title: Option<String>,  // Variant Title
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductCreateResult {
    pub product_id: Option<String>,
    pub handle: Option<String>,
    pub variant_id: Option<String>,     // Add variant_id to the result
    pub variant_title: Option<String>,  // Add varinat_title to result
}

pub async fn product_create(shop: &ShopifyCreds, product_input: ProductInput, rate_limiter: &RateLimiter)
-> Result<ProductCreateResult, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    productCreate($input: ProductInput!){
        productCreate(input: $input){
            product{
                id
                handle
                variants(first: 1) {
                    nodes {
                        id
                        title
                    }
                }
            }
            userErrors{
                field
                message
            }
        }
    }"#;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Input {
        input: ProductInput,
    }

    let new_product = Input { input: product_input };
    
    match shopify_graphql_request_with_retries::<ProductCreateResponse, Input>(gql_client, &mutation, Some(new_product), 3, rate_limiter).await {
        Ok((response, _)) => {
            
            // println!("{:?}", response);
            
            if let Some(product_create_response) = response.product_create {
                if let Some(new_product) = product_create_response.product {
                    // Extract the first variant ID
                    let variant_id = new_product.variants.clone()
                        .and_then(|variants| variants.nodes)
                        .and_then(|nodes| nodes.first().and_then(|variant| variant.id.clone()));
                     
                    let variant_title = new_product.variants
                        .and_then(|variants| variants.nodes)
                        .and_then(|nodes| nodes.first().and_then(|variant| variant.title.clone()));
                        
                    // Return the product and variant details
                    return Ok(ProductCreateResult {
                        product_id: new_product.id,
                        handle: new_product.handle,
                        variant_id,
                        variant_title,
                    });
                }
            }

            // If no product or variant is found, return an error
            Err(AppError::OtherError("Failed to create product or retrieve variant ID".to_string()))
        },
        Err(e) => {
            Err(AppError::OtherError(e.to_string()))
        }
    }
}