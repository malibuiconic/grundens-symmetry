use serde::{ Deserialize, Serialize };

// https://shopify.dev/docs/api/admin-graphql/2025-04/objects/App
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    #[serde(rename="apiKey")]
    pub api_key: Option<String>,
    #[serde(rename="developerName")]
    pub developer_name: Option<String>,
    pub handle: Option<String>,
    //.. other fields
}