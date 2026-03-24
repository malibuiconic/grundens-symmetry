#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use chrono::{ DateTime, Utc };

// https://shopify.dev/docs/api/admin-graphql/2025-01/objects/Image
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    #[serde(rename="altText")]
    pub alt_text: Option<String>,
    pub url: Option<String>,
}