#![allow(non_snake_case)]
// STOREFRONT API SPECIFIC FUNCTIONS - (Note uses private API key with storefront api scopes)
use serde::{ Deserialize, Serialize };
use crate::customer::graphQL::CountryCode;
use crate::customer::graphQL::MailingAddressInput;

pub mod cartCreate;            

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CartInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<AttributeInput>>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(rename="buyerIdentity")]
    // pub buyer_identity: Option<CartBuyerIdentityInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CartDeliveryInput>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(rename="discountCodes")]
    // pub discount_codes: Option<Vec<String>>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(rename="giftCardCodes")]
    // pub gift_card_codes: Option<Vec<String>>,           // case-insentive
    pub lines: Option<Vec<CartLineInput>>,
    // pub metafields: Option<Vec<CartInputMetafieldInput>>       
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartBuyerIdentityInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="companyLocationId")]
    pub company_location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="countryCode")]
    pub country_code: Option<CountryCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="customerAccessToken")]
    pub customer_access_token: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<CartPreferencesInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartLineInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<AttributeInput>>,
    #[serde(rename="merchandiseId")]
    pub merchandise: String,                           // Required: eg: gid://shopify/ProductVariant/50759211876668
    pub quantity: u32,                                 // default: 1
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="sellingPlanId")]
    pub selling_plan_id: Option<String>,               // Required with our Subs: eg: gid://shopify/SellingPlan/50759211876668
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartDeliveryInput {
    pub addresses: Option<Vec<CartSelectableAddressInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartSelectableAddressInput {
    pub address: CartAddressInput,                // required
    pub selected: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartAddressInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="copyFromCustomerAddressId")]
    pub copy_from_customer_address_id: Option<String>,         // GID Copy Customer AddressID
    #[serde(rename="deliveryAddress")]
    pub delivery_address: Option<MailingAddressInput>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartPreferencesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CartDeliveryPreferencesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartDeliveryPreferencesInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<CartDeliveryCoordinatesPreferencesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="deliveryMethod")]
    pub delivery_method: Option<PreferencesDeliveryMethodType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="pickupHandle")]
    pub pickup_handle: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PreferencesDeliveryMethodType {           // TODO: Expand proper ENUM handling - Luckily TF doesn't ship
    PICKUPPOINT,
    PICKUP,
    SHIPPING,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CartDeliveryCoordinatesPreferencesInput {
    #[serde(rename="countryCode")]
    pub country_code: CountryCode,               
    pub latitude: f64,                           // required
    pub longitude: f64,                          // required
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeInput {
    pub key: String,
    pub value: String,
}
