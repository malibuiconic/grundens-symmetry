use serde::{ Deserialize, Serialize };
use crate::money::MoneyInput;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShippingLineInput {
    #[serde(rename="priceWithCurrency")]
    pub price_with_currency: Option<MoneyInput>,
    #[serde(rename="shippingRateHandle")]
    pub shipping_rate_handle: Option<String>,
    pub title: Option<String>,
}