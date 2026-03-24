use serde::{Deserialize, Serialize};
use crate::money::MoneyBag;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxLine {
    #[serde(rename = "channelLiable")]
    pub channel_liable: Option<bool>,
    #[serde(rename="priceSet")]
    pub price_set: Option<MoneyBag>,
    pub rate: Option<f64>,
    #[serde(rename="ratePercentage")]
    pub rate_percentage: Option<f64>,
    pub source: Option<String>,
    pub title: Option<String>,
    // pub price: Option<Money>,   // deprecated
}
