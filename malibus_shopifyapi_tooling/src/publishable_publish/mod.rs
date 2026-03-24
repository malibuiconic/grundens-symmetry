#![allow(non_snake_case)]
use serde::{ Deserialize, Serialize };
use chrono::prelude::*;

pub mod publications;
pub mod publishablePublish;

// SalesChannels

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicationInput{
    #[serde(rename="publicationId")]
    pub publication_id: Option<String>,    // Id of the publication (so Online Store For instance)
    #[serde(rename="publishDate")]
    pub publish_date: Option<DateTime<Utc>>
}