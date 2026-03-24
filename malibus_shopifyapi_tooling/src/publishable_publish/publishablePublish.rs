use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::error::UserError;
use crate::AppError;
use crate::{RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries };
use super::PublicationInput;

// https://shopify.dev/docs/api/admin-graphql/2024-10/mutations/publishablepublish
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PublishablePublishResponse {
    #[serde(rename="publishablePublish")]
    publishable_publish: Option<PublishablePublish>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PublishablePublish {
    publishable: Option<Publishable>,
    shop: Option<serde_json::Value>,
    user_errors: Option<Vec<UserError>>  
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Shop {
    //.. 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Publishable {
    #[serde(rename="availablePublicationCount")]
    available_publication_count: Option<Count>,
    #[serde(rename="resourcePublicationCount")]
    resource_publication_count: Option<Count>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Count {
    count: Option<u32>
}

// Future adoption to a generic as this could be flexible on the product input and rather input too. 
pub async fn publishable_publish(shop: &ShopifyCreds, rate_limiter: &RateLimiter, publication: PublicationInput, product_gid: String)
-> Result<(), AppError>{
    
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mutation = r#"mutation
    publishablePublish($id: ID!, $input: [PublicationInput!]!) {
        publishablePublish(id: $id, input: $input) {
            publishable {
                availablePublicationsCount {
                    count
                }
                resourcePublicationsCount {
                    count
                }
            }
            shop {
                id
                name
                channelDefinitionsForInstalledChannels{
                   channelDefinitions {
                      channelName
                      handle
                   }
                   channelName
                }
            }
            userErrors {
                field
                message
            }
        }
    }"#;
    let pub_gid = match publication.publication_id {
        Some(gid) => gid,
        None => String::new()
    };
    let publish_product = json!({"id": product_gid, "input": { "publicationId": pub_gid }});
    // println!("{}", serde_json::to_string_pretty(&publish_product).unwrap());
    match shopify_graphql_request_with_retries::<PublishablePublishResponse, serde_json::Value>(gql_client, &mutation, Some(publish_product), 3, rate_limiter).await{
        Ok((response, extensions)) => {
           // println!("{:?}", response);
           if let Some(publishable_response) = response.publishable_publish {
              if let Some(user_errors) = publishable_response.user_errors {
                  println!("Publishable Publish Error: {:?}", user_errors)
              }
           }
           return Ok(())
        },
        Err(e) => { 
             Err(AppError::OtherError(e.to_string()))
        }
    }
}