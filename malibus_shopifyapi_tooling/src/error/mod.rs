use thiserror::Error;
use serde::{ Deserialize, Serialize };

#[derive(Error, Debug)]
pub enum AppError {
    // Shopify Errors
    #[error("GQL Client error when attempting query")]
    GraphQlErrorQueryAttempt(anyhow::Error),
    #[error("TAKEN")]
    MetaObjectDefinitionCreateError(anyhow::Error),

     // MongoDB Errors
     #[error("Unable to Connect to Mongo DB!")]
     MongoDBErrorConnection(anyhow::Error),
     #[error("No Object ID Found")]
     ErrorNoObjectID(anyhow::Error),
    
    // CUSTOM ERROR 
    #[error("Invalid API Error")]
    InvalidApiResponse(anyhow::Error),
    #[error("{0}")]
    OtherError(String),
    
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserError {
    pub code: Option<String>,
    pub field: Option<Vec<serde_json::Value>>,
    pub message: Option<String>,
}