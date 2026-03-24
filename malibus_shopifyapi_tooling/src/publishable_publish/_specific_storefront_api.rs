use serde::{Deserialize, Serialize};
use crate::AppError;
use crate::{RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries };

// https://shopify.dev/docs/api/admin-graphql/2025-01/queries/publications

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PublicationResponse {
    publications: Option<Publications>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Publications {
    nodes: Option<Vec<Publication>>                 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Publication {
    id: Option<String>,
    catalog: Option<Catalog>,                        // Catalog.title will be used once offically depricated 
    name: Option<String>,                            // For now have access to name field on node
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Catalog {
    title: Option<String>,
}

const STOREFRONT_API_IDENTIFIERS: [&str; 2] = ["storefront_api", "Storefront API"];

pub async fn get_publications(shop: &ShopifyCreds, rate_limiter: &RateLimiter)
-> Result<String, AppError>{
   
   let gql_client = ShopifyCreds::gql_client_builder(shop);
   let query = r#"query {
    publications(first: 5) {
      nodes {
        id
        name
        catalog {
           title
        }
      }
    }
   }"#;
   
   match shopify_graphql_request_with_retries::<PublicationResponse, ()>(gql_client, &query, None, 3, rate_limiter).await {
    Ok((response, _)) => {
        let publications = response.publications
            .and_then(|p| p.nodes)
            .ok_or_else(|| AppError::OtherError("No publications found".to_string()))?;

        for publication in publications {
            // First try to match by catalog title if available
            if let Some(catalog) = &publication.catalog {
                if let Some(title) = &catalog.title {
                    if STOREFRONT_API_IDENTIFIERS.contains(&title.as_str()) {
                        return publication.id
                            .ok_or_else(|| AppError::OtherError("Publication ID not found".to_string()));
                    }
                }
            }

            // Fall back to name matching
            if let Some(name) = &publication.name {
                if STOREFRONT_API_IDENTIFIERS.contains(&name.as_str()) {
                    return publication.id
                        .ok_or_else(|| AppError::OtherError("Publication ID not found".to_string()));
                }
            }
        }
        Err(AppError::OtherError("Storefront API publication not found".to_string()))
    },
    Err(e) => Err(AppError::OtherError(e.to_string()))
  }
}
