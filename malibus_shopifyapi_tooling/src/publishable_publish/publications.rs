use serde::{Deserialize, Serialize};
use crate::AppError;
use crate::{RateLimiter, ShopifyCreds, shopify_graphql_request_with_retries};
use crate::PageInfo;

pub const STOREFRONT_API_IDENTIFIERS: [&str; 2] = ["storefront_api", "Storefront API"];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicationResponse {
    publications: Option<Publications>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Publications {
    edges: Vec<PublicationEdge>,
    nodes: Vec<Publication>,
    pageInfo: PageInfo
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicationEdge {
    pub node: Publication,
    pub cursor: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Publication {
    pub id: String,
    pub name: Option<String>,
    pub catalog: Option<Catalog>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Catalog {
    pub title: Option<String>,
}

pub async fn get_publications(shop: &ShopifyCreds, rate_limiter: &RateLimiter) -> Result<Vec<Publication>, AppError> {
    let gql_client = ShopifyCreds::gql_client_builder(shop);
    let mut all_publications = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let cursor_param = match &cursor {
            Some(c) => format!(", after: \"{}\"", c),
            None => String::new(),
        };

        let query = format!(
            r#"query {{
                publications(first: 25{}) {{
                    edges {{
                        node {{
                            id
                            name
                            catalog {{
                                title
                            }}
                        }}
                        cursor
                    }}
                    nodes {{
                        id
                        name
                        catalog {{
                            title
                        }}
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                }}
            }}"#, 
            cursor_param
        );

        match shopify_graphql_request_with_retries::<PublicationResponse, ()>(
            gql_client.clone(), 
            &query, 
            None, 
            3, 
            rate_limiter
        ).await {
            Ok((response, _)) => {
                let publications = response.publications
                    .ok_or_else(|| AppError::OtherError("No publications found".to_string()))?;
                
                // Add logging to see what publications we're getting
                // println!("Found {} publications", publications.edges.len());
                for edge in &publications.edges {
                    /*
                    println!("Publication: id={}, name={:?}, catalog_title={:?}", 
                        edge.node.id,
                        edge.node.name,
                        edge.node.catalog.as_ref().and_then(|c| c.title.as_ref())
                    );*/
                }

                // Add current page's publications to our collection
                for edge in publications.edges {
                    all_publications.push(edge.node);
                }

                // Check if we need to continue pagination
                if publications.pageInfo.has_next_page.unwrap_or(false) {
                    cursor = publications.pageInfo.end_cursor;
                } else {
                    break;
                }
            },
            Err(e) => return Err(AppError::OtherError(e.to_string()))
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    if all_publications.is_empty() {
        //println!("No publications found for shop");
    }

    Ok(all_publications)
}