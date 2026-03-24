use serde::{Deserialize, Serialize};
use crate::AppError;
use crate::{RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries };

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PublicationResponse {
    publications: Option<Publications>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Publications {
    nodes: Option<Vec<Publication>>                 // Typically the Online Store would be the first one!
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Publication {
    id: Option<String>
}


pub async fn get_publications(shop: &ShopifyCreds, rate_limiter: &RateLimiter)
-> Result<String, AppError>{
   
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let query = r#"query
   publications(first: 5){
      nodes {
         id
      }
   }"#;
   
   match shopify_graphql_request_with_retries::<PublicationResponse, ()>(gql_client, &query, None, 3, rate_limiter).await{
        Ok((response, extensions)) => {
            if let Some(publications) = &response.publications {
                if let Some(nodes) = &publications.nodes {
                    for (index, node) in nodes.iter().enumerate() {
                        if index == 0 {
                            // Online Store Publication ID (SalesChannel ID)
                            if let Some (gid) = node.id.clone() {
                                return Ok(gid) 
                            }
                        }
                    }
                }
            }
            return Ok(String::from(""))
        },
        Err(e) => { 
            Err(AppError::OtherError(e.to_string()))
        }
    }
}
