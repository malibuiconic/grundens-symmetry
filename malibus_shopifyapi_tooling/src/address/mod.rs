use serde::{Deserialize, Serialize};
use crate::customer::graphQL::CountryCode;

// https://shopify.dev/docs/api/admin-graphql/2025-04/objects/MailingAddress
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MailingAddress {
    #[serde(rename="address1")]
    pub address1: Option<String>,
    #[serde(rename="address2")]
    pub address2: Option<String>,
    pub city: Option<String>,
    pub company: Option<String>,
    pub country: Option<String>,
    #[serde(rename="countryCodeV2")]
    pub country_code: Option<CountryCode>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub id: Option<String>,
    pub province: Option<String>,
    #[serde(rename="provinceCode")]
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

impl MailingAddress {
    pub fn normalize(&self) -> NormalizedAddress {
        NormalizedAddress {
            street: self.address1.as_ref()
                .map(|s| s.to_uppercase().trim()
                    .replace(".", "")
                    .replace(",", "")
                    .replace("#", " ")
                    .replace("  ", " ") // Clean double spaces
                    .trim().to_string()),
            city: self.city.as_ref()
                .map(|s| s.to_uppercase().trim().to_string()),
            state: self.province.as_ref()
                .map(|s| s.to_uppercase().trim().to_string()),
            state_code: self.province_code.as_ref()
                .map(|s| s.to_uppercase().trim().to_string()),
            postal_code: self.zip.as_ref()
                .map(|s| s.to_uppercase().trim().replace(" ", "").to_string()),
            country: self.country.as_ref()
                .map(|s| s.to_uppercase().trim().to_string()),
            country_code: self.country_code.as_ref()
                .map(|code| match code {
                    CountryCode::US => "US".to_string(),
                    CountryCode::CA => "CA".to_string(),
                    _ => code.to_string().to_uppercase(),
                }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NormalizedAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub state_code: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
}