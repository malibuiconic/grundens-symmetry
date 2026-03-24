use serde::{ Deserialize, Serialize };
use chrono::{ DateTime, Utc };
use crate::PageInfo;
use crate::customer::Customer;
use crate::customer::graphQL::CountryCode;
use crate::subscriptions::SubscriptionContractConnection;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentTermsInput {
    #[serde(rename="paymentSchedules")]
    pub payment_schedules: Option<PaymentScheduleInput>,
    #[serde(rename="paymentTermsTemplateId")]
    pub payment_terms_template_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentScheduleInput {
    #[serde(rename="dueAt")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(rename="issuedAt")]
    pub issued_at: Option<DateTime<Utc>>,
}

// Connections:
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerPaymentMethodConnection {
    pub edges: Option<Vec<CustomerPaymentMethodEdge>>,
    // pub nodes: Option<Vec<CustomerPaymentMethod>>,
    #[serde(rename="pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerPaymentMethodEdge {
    pub node: Option<CustomerPaymentMethod>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerPaymentMethod {
    pub customer: Option<Customer>,
    pub id: Option<String>,
    pub instrument: Option<CustomerPaymentInstrument>,
    #[serde(rename="revokedAt")]
    pub revoked_at: Option<DateTime<Utc>>,
    // #[serde(rename="revokedReason")]
    // pub revoked_reason: Option<CustomerPaymentMethodRevocationReason>,
    #[serde(rename="subscriptionContracts")]
    pub subscription_contracts: Option<SubscriptionContractConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerPaymentInstrument {
    #[serde(rename="CustomerCreditCard")]
    pub customer_credit_card: Option<CustomerCreditCard>,
    // #[serde(rename="CustomerPaypalBillingAggreement")]
    // pub customer_paypal_billing_aggreement: Option<CustomerPaypalBillingAggreement>
    // #[serde(rename="CustomerShopPayAgreement")]
    // pub customer_shoppay_aggreement: Option<CustomerShopPayAgreement>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerCreditCard {
    #[serde(rename="billingAddress")]
    pub billing_address: Option<CustomerCreditCardBillingAddress>,
    pub brand: Option<String>,
    #[serde(rename="expiresSoon")]
    pub expires_soon: Option<bool>,
    #[serde(rename="expiryMonth")]
    pub expiry_month: Option<u32>,
    #[serde(rename="expiryYear")]
    pub expiry_year: Option<u32>,
    #[serde(rename="firstDigits")]
    pub first_digits: Option<String>,
    #[serde(rename="isRevocable")]
    pub is_revocable: Option<bool>,
    #[serde(rename="lastDigits")]
    pub last_digits: Option<String>,
    #[serde(rename="maskedNumber")]
    pub masked_number: Option<String>,
    pub name: Option<String>,
    pub source: Option<String>,
    #[serde(rename="virtualLastDigits")]
    pub virtual_last_digits: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerCreditCardBillingAddress {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    #[serde(rename="countryCode")]
    pub country_code: Option<CountryCode>,
    #[serde(rename="firstName")]
    pub first_name: Option<String>,
    #[serde(rename="lastName")]
    pub last_name: Option<String>,
    pub province: Option<String>,
    #[serde(rename="provinceCode")]
    pub province_code: Option<String>,
    pub zip: Option<String>,
}

