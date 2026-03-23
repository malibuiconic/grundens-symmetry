use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopifyStore {
    pub id: Option<i64>,
    pub shop_domain: String,
    pub access_token: String,
    pub scope: String,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}
